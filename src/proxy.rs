//! The MITM proxy Tinman runs between a sandboxed target and the network.
//!
//! hudsucker is the maintained MITM HTTP/HTTPS proxy and har is the HAR
//! serialization library this seam is built on; what stays ours is the policy
//! that decides which fault is injected and whether an exchange is recorded
//! to, or replayed from, a HAR file.

use crate::sandbox::{Egress, Fault};
use hudsucker::{
    Body, HttpContext, HttpHandler, RequestOrResponse,
    certificate_authority::RcgenAuthority,
    hyper::{
        Request, Response, StatusCode,
        body::{Body as HttpBody, Bytes},
        header::{AUTHORIZATION, HeaderValue},
    },
    rcgen::{CertificateParams, Issuer, KeyPair},
    rustls::crypto::aws_lc_rs,
};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// A handle to one running proxy: the port it bound, and the credential slot
/// its own handler reads. A sandbox backend prepares a launch from its
/// specification alone, which names no proxy, so the caller that started the
/// proxy hands this over: the launch then bridges the exact port that proxy
/// bound, since bridging every host port would let a direct dial through as
/// well, and stages the credential on that same proxy rather than on whichever
/// one started last.
///
/// @planks("the target requests the allowed host directly by address")
/// @planks("the target prints its whole environment and then calls the provider")
#[derive(Debug, Clone)]
pub struct ProxyLink {
    addr: SocketAddr,
    credential: Arc<Mutex<Option<String>>>,
}

impl ProxyLink {
    /// Where the linked proxy listens, for a sandbox backend bridging a
    /// `proxy` launch to reach.
    ///
    /// @planks("the target requests the allowed host directly by address")
    pub fn address(&self) -> SocketAddr {
        self.addr
    }

    /// Hold `credential` for the linked proxy to attach to the exchange on the
    /// sandboxed target's behalf, since the target's own environment never
    /// carries it.
    ///
    /// @planks("the target prints its whole environment and then calls the provider")
    pub fn hold_credential(&self, credential: Option<String>) {
        *self
            .credential
            .lock()
            .expect("the held credential is not poisoned") = credential;
    }
}

/// A running proxy instance: where it listens, the credential it holds for the
/// sandboxed target's exchange, and what stops it.
#[derive(Debug)]
pub struct Proxy {
    addr: SocketAddr,
    credential: Arc<Mutex<Option<String>>>,
    stop: Option<tokio::sync::oneshot::Sender<()>>,
    task: Option<tokio::task::JoinHandle<()>>,
}

impl Proxy {
    /// Start a proxy under the given egress policy, on a loopback port the
    /// operating system chooses.
    ///
    /// @planks("a request crosses the proxy and is answered")
    /// @planks("a request crosses the proxy")
    /// @planks("the same request through the proxy is answered")
    pub fn start(egress: &Egress) -> Result<Proxy, String> {
        let listener = std::net::TcpListener::bind("127.0.0.1:0")
            .map_err(|e| format!("the proxy could not bind a loopback port: {e}"))?;
        let addr = listener
            .local_addr()
            .map_err(|e| format!("the proxy could not read its bound address: {e}"))?;
        let credential = Arc::new(Mutex::new(None));

        if egress.fault.as_ref().is_some_and(|fault| fault.offline) {
            // Nothing accepts on this port once the listener is dropped, so a
            // client that connects afterwards meets the refusal the offline
            // fault names, with no handler ever in the loop.
            drop(listener);
            return Ok(Proxy {
                addr,
                credential,
                stop: None,
                task: None,
            });
        }

        listener
            .set_nonblocking(true)
            .map_err(|e| format!("the proxy listener could not be made non-blocking: {e}"))?;
        let listener = tokio::net::TcpListener::from_std(listener)
            .map_err(|e| format!("the proxy listener could not join the runtime: {e}"))?;

        let key_pair = KeyPair::generate()
            .map_err(|e| format!("the proxy's throwaway CA key could not be generated: {e}"))?;
        let issuer = Issuer::new(CertificateParams::default(), key_pair);
        let ca = RcgenAuthority::new(issuer, 1, aws_lc_rs::default_provider());

        let (stop, done) = tokio::sync::oneshot::channel();

        let built = hudsucker::Proxy::builder()
            .with_listener(listener)
            .with_ca(ca)
            .with_rustls_connector(aws_lc_rs::default_provider())
            .with_http_handler(EgressHandler::new(egress, Arc::clone(&credential)))
            .with_graceful_shutdown(async {
                let _ = done.await;
            })
            .build()
            .map_err(|e| format!("the proxy could not be built: {e}"))?;

        let task = tokio::spawn(async move {
            let _ = built.start().await;
        });

        Ok(Proxy {
            addr,
            credential,
            stop: Some(stop),
            task: Some(task),
        })
    }

    /// Where the proxy listens, in the form a client configures as its proxy
    /// address.
    ///
    /// @planks("a request crosses the proxy")
    pub fn address(&self) -> SocketAddr {
        self.addr
    }

    /// A handle to this proxy, for the sandbox backend whose launch routes
    /// egress through this one rather than through any other the process is
    /// running.
    ///
    /// @planks("the target requests the allowed host directly by address")
    /// @planks("the target prints its whole environment and then calls the provider")
    pub fn link(&self) -> ProxyLink {
        ProxyLink {
            addr: self.addr,
            credential: Arc::clone(&self.credential),
        }
    }

    /// Stop the proxy. A signal is enough: every scenario that stops a proxy
    /// has already read everything its run produced.
    ///
    /// @planks("a HAR file carrying one recorded exchange")
    pub fn stop(mut self) {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        self.task.take();
    }
}

/// What the egress policy does with one request: inject a fault, record a
/// real exchange, or answer from a prior recording.
#[derive(Clone)]
struct EgressHandler {
    fault: Option<Fault>,
    har: Option<Arc<Mutex<HarState>>>,
    credential: Arc<Mutex<Option<String>>>,
    pending: Option<(String, String)>,
}

/// Whether a HAR-backed egress is capturing new exchanges or answering from
/// ones it already carries, decided once by the file's presence at start.
enum HarState {
    Recording {
        path: String,
        log: Box<har::v1_2::Log>,
    },
    Replaying {
        entries: Vec<har::v1_2::Entries>,
    },
}

impl EgressHandler {
    fn new(egress: &Egress, credential: Arc<Mutex<Option<String>>>) -> EgressHandler {
        let har = egress.har.as_ref().map(|path| {
            let state = if std::path::Path::new(path).exists() {
                HarState::Replaying {
                    entries: read_entries(path),
                }
            } else {
                HarState::Recording {
                    path: path.clone(),
                    log: Box::default(),
                }
            };
            Arc::new(Mutex::new(state))
        });
        EgressHandler {
            fault: egress.fault.clone(),
            har,
            credential,
            pending: None,
        }
    }
}

impl HttpHandler for EgressHandler {
    /// Attach the credential a sandbox backend staged on this proxy, if any,
    /// before this exchange is forwarded, so the upstream sees it on the
    /// target's behalf without the target ever holding it itself.
    ///
    /// @planks("the request the upstream received carries the credential")
    async fn handle_request(
        &mut self,
        _ctx: &HttpContext,
        mut req: Request<Body>,
    ) -> RequestOrResponse {
        let held = self
            .credential
            .lock()
            .expect("the held credential is not poisoned")
            .clone();
        if let Some(credential) = held {
            req.headers_mut().insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {credential}"))
                    .expect("the held credential forms a valid header value"),
            );
        }

        if let Some(fault) = &self.fault {
            if let Some(status) = fault.status {
                return RequestOrResponse::Response(
                    Response::builder()
                        .status(status)
                        .body(Body::empty())
                        .expect("the injected-status response builds"),
                );
            }
            if let Some(delay) = fault.latency.as_deref().and_then(parse_latency) {
                tokio::time::sleep(delay).await;
            }
        }

        if let Some(har) = &self.har {
            let method = req.method().to_string();
            let url = req.uri().to_string();
            let mut state = har.lock().expect("the HAR state lock is not poisoned");
            match &mut *state {
                HarState::Replaying { entries } => {
                    let matched = entries
                        .iter()
                        .find(|entry| entry.request.method == method && entry.request.url == url);
                    return RequestOrResponse::Response(match matched {
                        Some(entry) => Response::builder()
                            .status(entry.response.status as u16)
                            .body(Body::from(
                                entry.response.content.text.clone().unwrap_or_default(),
                            ))
                            .expect("the replayed response builds"),
                        None => Response::builder()
                            .status(StatusCode::BAD_GATEWAY)
                            .body(Body::from(format!(
                                "no recorded entry matches {method} {url}"
                            )))
                            .expect("the unmatched-entry response builds"),
                    });
                }
                HarState::Recording { .. } => {
                    self.pending = Some((method, url));
                }
            }
        }

        RequestOrResponse::Request(req)
    }

    async fn handle_response(&mut self, _ctx: &HttpContext, res: Response<Body>) -> Response<Body> {
        let Some((method, url)) = self.pending.take() else {
            return res;
        };
        let Some(har) = &self.har else {
            return res;
        };
        let (parts, body) = res.into_parts();
        let bytes = collect(body).await;
        {
            let mut state = har.lock().expect("the HAR state lock is not poisoned");
            if let HarState::Recording { path, log } = &mut *state {
                log.entries
                    .push(entry(method, url, parts.status.as_u16(), &bytes));
                write_har(path, log);
            }
        }
        Response::from_parts(parts, Body::from(bytes.to_vec()))
    }
}

/// Read a whole body into memory, so a recorded exchange's response can be
/// persisted and a forwarded one can be replayed byte for byte.
async fn collect(body: Body) -> Bytes {
    let mut body = std::pin::pin!(body);
    let mut collected = Vec::new();
    while let Some(Ok(frame)) = std::future::poll_fn(|cx| body.as_mut().poll_frame(cx)).await {
        if let Ok(data) = frame.into_data() {
            collected.extend_from_slice(&data);
        }
    }
    Bytes::from(collected)
}

/// Parse a duration such as `2s` into the delay it names.
fn parse_latency(value: &str) -> Option<Duration> {
    value
        .strip_suffix('s')
        .and_then(|seconds| seconds.parse::<u64>().ok())
        .map(Duration::from_secs)
}

/// Build the HAR entry one recorded exchange contributes.
fn entry(method: String, url: String, status: u16, body: &[u8]) -> har::v1_2::Entries {
    har::v1_2::Entries {
        request: har::v1_2::Request {
            method,
            url,
            http_version: "HTTP/1.1".to_string(),
            headers_size: -1,
            body_size: 0,
            ..Default::default()
        },
        response: har::v1_2::Response {
            status: status as i64,
            http_version: "HTTP/1.1".to_string(),
            content: har::v1_2::Content {
                size: body.len() as i64,
                mime_type: Some("application/octet-stream".to_string()),
                text: Some(String::from_utf8_lossy(body).into_owned()),
                ..Default::default()
            },
            headers_size: -1,
            body_size: body.len() as i64,
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Overwrite the recording at `path` with `log`.
fn write_har(path: &str, log: &har::v1_2::Log) {
    let document = har::Har {
        log: har::Spec::V1_2(log.clone()),
    };
    let json = har::to_json(&document).expect("the HAR document serializes");
    std::fs::write(path, json).expect("the HAR recording is writable");
}

/// Read the entries a HAR recording at `path` already carries.
fn read_entries(path: &str) -> Vec<har::v1_2::Entries> {
    let document = har::from_path(path)
        .unwrap_or_else(|e| panic!("the HAR recording {path} does not parse: {e}"));
    match document.log {
        har::Spec::V1_2(log) => log.entries,
        har::Spec::V1_3(_) => {
            panic!("the HAR recording {path} is version 1.3, but only 1.2 is ever written")
        }
    }
}

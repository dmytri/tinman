//! Inference configuration and the requests Tinman sends.
//!
//! Tinman speaks the OpenAI-compatible chat-completions protocol. The
//! credential, the endpoint and the model are configuration, read from the
//! environment or a dotenv file, so reaching a different compatible provider is
//! a configuration change rather than a code change. Inference unavailable is a
//! normal degraded mode: a missing credential, an unreachable provider or a
//! rejected credential all report unavailable.

use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

/// The endpoint Tinman addresses when none is configured.
const DEFAULT_BASE_URL: &str = "https://openrouter.ai/api/v1";

/// The model Tinman addresses when none is configured.
const DEFAULT_MODEL: &str = "deepseek/deepseek-v4-flash";

/// The acronym is cosmetic and optimizes for novelty.
const ACRONYM_TEMPERATURE: f64 = 1.4;

/// The assistant proposes commands a user may run, so it optimizes for
/// correctness, determinism and instruction following.
const ASSISTANT_TEMPERATURE: f64 = 0.1;

/// Reading a screen is a reading task, so the terminal object model optimizes
/// for a faithful reading rather than for novelty.
const TOM_TEMPERATURE: f64 = 0.1;

/// What the engine is asked when it reads a screen into a terminal object
/// model.
const TOM_INSTRUCTION: &str = concat!(
    "Read the terminal screen below and reply with a JSON terminal object model ",
    "and nothing else. The object carries the integer keys \"rows\" and \"cols\" ",
    "and the key \"root\", a region. A region carries \"role\", one of application, ",
    "region, menu, menuitem, list, listitem, button, textbox, status, log, ",
    "article; \"name\" and ",
    "\"text\", each a string or null; \"selected\", a boolean; \"rect\", with the ",
    "integer keys \"x\", \"y\", \"width\" and \"height\"; and \"children\", an ",
    "array of regions.\n",
);

/// The inference configuration a run resolved.
///
/// @planks("Tinman resolves its inference credential")
#[derive(Debug, Clone)]
pub struct Settings {
    /// The provider credential, absent when nothing configures one.
    pub api_key: Option<String>,
    /// The OpenAI-compatible endpoint Tinman addresses.
    pub base_url: String,
    /// The model Tinman addresses.
    pub model: String,
}

impl Settings {
    /// Resolve the configuration from `env` and a dotenv file in `dir`. The
    /// environment overrides the file.
    ///
    /// @planks("Tinman resolves its inference credential")
    pub fn resolve(env: &BTreeMap<String, String>, dir: &Path) -> Settings {
        let mut values: BTreeMap<String, String> = BTreeMap::new();
        let dotenv = dir.join(".env");
        if dotenv.exists() {
            let entries = dotenvy::from_path_iter(&dotenv)
                .unwrap_or_else(|e| panic!("dotenv file {} unreadable: {e}", dotenv.display()));
            for entry in entries {
                let (key, value) = entry.unwrap_or_else(|e| {
                    panic!("dotenv file {} did not parse: {e}", dotenv.display())
                });
                values.insert(key, value);
            }
        }
        for (key, value) in env {
            values.insert(key.clone(), value.clone());
        }
        Settings {
            api_key: values.get("TINMAN_API_KEY").cloned(),
            base_url: values
                .get("TINMAN_BASE_URL")
                .cloned()
                .unwrap_or_else(|| DEFAULT_BASE_URL.to_string()),
            model: values
                .get("TINMAN_MODEL")
                .cloned()
                .unwrap_or_else(|| DEFAULT_MODEL.to_string()),
        }
    }

    /// Resolve the configuration a running Tinman sees: the process environment
    /// and a dotenv file in the working directory.
    ///
    /// @planks("the operator runs {string} in an interactive terminal")
    pub fn from_process() -> Settings {
        let env: BTreeMap<String, String> = std::env::vars().collect();
        let dir = std::env::current_dir().expect("the working directory is readable");
        Settings::resolve(&env, &dir)
    }
}

/// One chat-completions request: where it goes, which model answers it, how it
/// samples and what it asks.
///
/// @planks("an inference request is built")
#[derive(Debug, Clone)]
pub struct Request {
    base_url: String,
    model: String,
    authorization: Option<String>,
    temperature: f64,
    prompt: String,
}

impl Request {
    /// The provider endpoint this request addresses.
    ///
    /// @planks("the request addresses {string} with the model {string}")
    pub fn address(&self) -> &str {
        &self.base_url
    }

    /// The absolute chat-completions endpoint this request is sent to.
    fn url(&self) -> String {
        format!("{}/chat/completions", self.base_url.trim_end_matches('/'))
    }

    /// The model this request names.
    ///
    /// @planks("the request addresses {string} with the model {string}")
    /// @planks("both requests name the model {string}")
    pub fn model(&self) -> &str {
        &self.model
    }

    /// The sampling temperature this request carries.
    ///
    /// @planks("the request temperature is {float}")
    pub fn temperature(&self) -> f64 {
        self.temperature
    }

    /// The authorization header this request carries, absent when no credential
    /// is configured.
    ///
    /// @planks("the request carries the authorization header {string}")
    pub fn authorization(&self) -> Option<&str> {
        self.authorization.as_deref()
    }
}

/// The wire form of a request: the endpoint it addresses, the credential it
/// carries, the model it names, how it samples and the chat messages it sends.
///
/// @planks("it conforms to the {string} schema in {string}")
impl serde::Serialize for Request {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut wire = serializer.serialize_struct("Request", 5)?;
        wire.serialize_field("url", &self.url())?;
        wire.serialize_field("authorization", &self.authorization)?;
        wire.serialize_field("model", &self.model)?;
        wire.serialize_field("temperature", &self.temperature)?;
        wire.serialize_field(
            "messages",
            &[WireMessage {
                role: "user",
                content: &self.prompt,
            }],
        )?;
        wire.end()
    }
}

/// One chat message on the wire.
///
/// @planks("it conforms to the {string} schema in {string}")
#[derive(serde::Serialize)]
struct WireMessage<'a> {
    role: &'a str,
    content: &'a str,
}

/// The acronym request: the bundled skill's name and description, sampled at a
/// very high temperature.
///
/// @planks("the acronym request is built")
/// @planks("the acronym request and the assistant request are built")
pub fn acronym_request(settings: &Settings) -> Request {
    request(
        settings,
        ACRONYM_TEMPERATURE,
        crate::skill::acronym_context(),
    )
}

/// The assistant request: the whole bundled skill body and the operator's
/// question, sampled at a low temperature.
///
/// @planks("the assistant request is built")
/// @planks("the acronym request and the assistant request are built")
pub fn assistant_request(settings: &Settings, question: &str) -> Request {
    let prompt = format!("{}\n{question}", crate::skill::assistant_context());
    request(settings, ASSISTANT_TEMPERATURE, prompt)
}

/// Build a request against the configured settings. The credential becomes a
/// bearer token, absent when no credential is configured.
///
/// @planks("the request carries the authorization header {string}")
/// @planks("the request carries no authorization header")
fn request(settings: &Settings, temperature: f64, prompt: String) -> Request {
    Request {
        base_url: settings.base_url.clone(),
        model: settings.model.clone(),
        authorization: settings.api_key.as_ref().map(|key| format!("Bearer {key}")),
        temperature,
        prompt,
    }
}

/// The acronym expansion the configured provider generated, absent when no
/// credential is configured, when the provider cannot be reached, when it
/// rejects the credential, or when it generates nothing.
///
/// @planks("the operator runs {string} in an interactive terminal")
pub fn expansion(settings: &Settings) -> Option<String> {
    settings.api_key.as_ref()?;
    let generated = complete(&acronym_request(settings))?;
    let generated = generated.trim();
    if generated.is_empty() {
        None
    } else {
        Some(generated.to_string())
    }
}

/// The reply the configured provider generated for the assistant's question,
/// absent when the provider cannot be reached, rejects the request, or answers
/// with nothing.
///
/// @planks("the operator asks {string}")
pub fn assistant_completion(settings: &Settings, question: &str) -> Option<String> {
    complete(&assistant_request(settings, question))
}

/// The terminal object model the configured provider read from the screen it
/// was shown, absent when no credential is configured, when the provider cannot
/// be reached, when it rejects the credential, or when it answers with nothing.
///
/// @planks("the terminal object model is inferred")
pub fn tom_completion(settings: &Settings, screen: &str) -> Option<String> {
    settings.api_key.as_ref()?;
    let prompt = format!("{TOM_INSTRUCTION}{screen}");
    complete(&request(settings, TOM_TEMPERATURE, prompt))
}

/// Whether the configured provider answers.
///
/// @planks("Tinman checks whether inference is available")
pub fn is_available(settings: &Settings) -> bool {
    expansion(settings).is_some()
}

/// Send a request to the configured provider and return the content it
/// generated. Absent when the provider cannot be reached, rejects the request,
/// or answers with nothing.
fn complete(request: &Request) -> Option<String> {
    let url = request.url();
    let body = format!(
        r#"{{"model":{},"temperature":{},"messages":[{{"role":"user","content":{}}}]}}"#,
        json_string(&request.model),
        request.temperature,
        json_string(&request.prompt)
    );
    let mut call = ureq::post(&url).header("Content-Type", "application/json");
    if let Some(authorization) = &request.authorization {
        call = call.header("Authorization", authorization);
    }
    let mut response = call.send(&body).ok()?;
    let text = response.body_mut().read_to_string().ok()?;
    let completion: Completion = serde_yaml::from_str(&text).ok()?;
    completion
        .choices
        .into_iter()
        .next()
        .map(|choice| choice.message.content)
}

/// A JSON string literal carrying `value`.
fn json_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for character in value.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// The provider's reply. JSON is a subset of YAML, so the rigged YAML
/// deserializer reads it.
#[derive(Debug, Deserialize)]
struct Completion {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Debug, Deserialize)]
struct Message {
    content: String,
}

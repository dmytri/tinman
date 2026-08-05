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
const DEFAULT_MODEL: &str = "deepseek/deepseek-v4-flash-0731";

/// The source tldr pages are read from when none is configured: the
/// tldr-pages project's own raw markdown.
const DEFAULT_TLDR_BASE_URL: &str = "https://raw.githubusercontent.com/tldr-pages/tldr/main/pages";

/// The ceiling on an availability probe, from connection to the last byte of
/// the answer. A provider that accepts the connection and then withholds its
/// answer ends the call only on a ceiling, so every call carries one. A probe
/// asks only whether the provider answers, so it is bounded tightly and a slow
/// answer counts as no answer.
pub(crate) const PROBE_CEILING: std::time::Duration = std::time::Duration::from_secs(20);

/// The ceiling on a generation call, from connection to the last byte of the
/// answer. A generation call asks a model to produce a structured document,
/// which legitimately takes tens of seconds, so the probe's tight bound would
/// truncate real work and report absence where the provider was answering. The
/// bound clears the slow tail of a real screen reading rather than its median,
/// because a ceiling sized to the median reports absence on the tail.
pub(crate) const GENERATION_CEILING: std::time::Duration = std::time::Duration::from_secs(110);

/// The ceiling on the tagline generation an interactive help waits for. The
/// expansion is cosmetic, so a provider that has not answered by then is read as
/// no answer and the unavailable notice fills the line. The operator asked for
/// the help and the prompt beneath it, and a real model call takes tens of
/// seconds, so waiting the generation ceiling out holds a blank terminal in
/// front of them for the whole of it.
pub(crate) const TAGLINE_CEILING: std::time::Duration = std::time::Duration::from_secs(5);

/// The acronym is cosmetic and optimizes for novelty.
const ACRONYM_TEMPERATURE: f64 = 1.4;

/// The letters an expansion must spell, in order, to be an acronym at all.
const ACRONYM_LETTERS: &str = "tinman";

/// How many times a provider is asked for an expansion that spells the name.
/// An expansion that misses is replaced rather than dressed up, and the ask is
/// counted as well as timed so a provider that keeps missing settles rather
/// than spending the whole ceiling in front of an operator.
const ACRONYM_ASKS: usize = 2;

/// The assistant proposes commands a user may run, so it optimizes for
/// correctness, determinism and instruction following.
const ASSISTANT_TEMPERATURE: f64 = 0.1;

/// Reading a screen is a reading task, so the terminal object model optimizes
/// for a faithful reading rather than for novelty.
const TOM_TEMPERATURE: f64 = 0.1;

/// What the engine is asked when it reads a screen into a terminal object
/// model. The schema alone leaves an engine free to put every line it reads
/// into `text` and leave every `name` null, which is a reading that adds
/// nothing the deterministic pass had not already read, so the instruction
/// says what a name is and where the screen shows one. It says as well that
/// the name belongs on the region holding the labelled content, because a
/// reply that hangs the label on a wrapper of its own leaves the region the
/// deterministic reading built still unnamed.
///
/// @planks("the operator inspects that program")
const TOM_INSTRUCTION: &str = concat!(
    "Read the terminal screen below and reply with a JSON terminal object model ",
    "and nothing else. The object carries the integer keys \"rows\" and \"cols\" ",
    "and the key \"root\", a region. A region carries \"role\", one of application, ",
    "region, menu, menuitem, list, listitem, button, textbox, status, log, ",
    "article; \"name\" and ",
    "\"text\", each a string or null; \"selected\", a boolean; \"rect\", with the ",
    "integer keys \"x\", \"y\", \"width\" and \"height\"; and \"children\", an ",
    "array of regions.\n",
    "A region's \"name\" is the label the screen itself shows for that region. A ",
    "line of text standing above a bordered pane labels what the pane holds, and ",
    "a title drawn into the border labels it the same way. Carry that text as ",
    "the \"name\" of the list, table, menu or pane drawn there, and report the ",
    "label as no region of its own. Only \"root\" covers the whole screen, and no ",
    "region exists merely to hold another. Leave \"name\" null where the screen ",
    "shows no label for the region.\n",
);

/// What the engine is told the page in front of the screen is. A tldr page
/// describes what a command is for in the words its own community chose, which
/// is the vocabulary a good name comes from, and it describes some version of
/// the program rather than the screen in hand.
const TOM_PAGE_INSTRUCTION: &str = concat!(
    "The community documentation below describes the program on the screen. ",
    "Take its vocabulary when naming regions, and name nothing the screen does ",
    "not itself show.\n",
);

/// The inference configuration a run resolved.
///
/// @planks("Tinman resolves its inference credential")
/// @planks("the page source is read")
#[derive(Debug, Clone)]
pub struct Settings {
    /// The provider credential, absent when nothing configures one.
    pub api_key: Option<String>,
    /// The OpenAI-compatible endpoint Tinman addresses.
    pub base_url: String,
    /// The model Tinman addresses.
    pub model: String,
    /// The source a naming pass reads a program's tldr page from.
    pub tldr_base_url: String,
}

impl Settings {
    /// Resolve the configuration from `env` and a dotenv file in `dir`. The
    /// environment overrides the file.
    ///
    /// @planks("Tinman resolves its inference credential")
    /// @planks("the page source is read")
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
            tldr_base_url: values
                .get("TINMAN_TLDR_BASE_URL")
                .cloned()
                .unwrap_or_else(|| DEFAULT_TLDR_BASE_URL.to_string()),
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

/// The most recent exchanges the compacted transcript carries whole, both
/// question and answer, before an older one keeps only its question.
const WHOLE_WINDOW: usize = 17;

/// The character ceiling the compacted transcript is kept under. The bundled
/// skill is fixed and dwarfs it, so this budget covers the transcript alone.
const TRANSCRIPT_BUDGET: usize = 120_000;

/// One turn already put to the model in the current session: the question
/// asked and, while the compacted transcript still carries it, the answer
/// received.
#[derive(Debug, Clone)]
pub struct Exchange {
    /// The question this turn asked.
    pub question: String,
    /// The answer this turn received, absent once compaction has dropped it.
    pub answer: Option<String>,
}

impl Exchange {
    /// An exchange carrying `question` and the `answer` it received.
    ///
    /// @planks("the assistant request carries the earlier question {string}")
    /// @planks("the assistant request carries the earlier answer {string}")
    pub fn new(question: &str, answer: &str) -> Exchange {
        Exchange {
            question: question.to_string(),
            answer: Some(answer.to_string()),
        }
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
    messages: Vec<(String, String)>,
    /// Whether the provider is asked to reason, absent where the request sets
    /// no reasoning control.
    reasoning: Option<bool>,
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
/// carries, and the body the provider receives. The endpoint and the
/// credential are transport rather than payload, so they sit beside the body
/// instead of inside it.
///
/// @planks("it conforms to the {string} schema in {string}")
impl serde::Serialize for Request {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut wire = serializer.serialize_struct("Request", 3)?;
        wire.serialize_field("url", &self.url())?;
        wire.serialize_field("authorization", &self.authorization)?;
        let messages: Vec<WireMessage> = self
            .messages
            .iter()
            .map(|(role, content)| WireMessage { role, content })
            .collect();
        wire.serialize_field(
            "body",
            &WireBody {
                model: &self.model,
                temperature: self.temperature,
                messages,
                reasoning: self.reasoning,
            },
        )?;
        wire.end()
    }
}

/// The JSON document sent as the request body: the whole of what the provider
/// receives, nested under the wire form's `body` field.
///
/// @planks("it conforms to the {string} schema in {string}")
/// @planks("the request disables reasoning")
struct WireBody<'a> {
    model: &'a str,
    temperature: f64,
    messages: Vec<WireMessage<'a>>,
    reasoning: Option<bool>,
}

impl<'a> serde::Serialize for WireBody<'a> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut wire = serializer.serialize_struct("WireBody", 4)?;
        wire.serialize_field("model", self.model)?;
        wire.serialize_field("temperature", &self.temperature)?;
        wire.serialize_field("messages", &self.messages)?;
        match self.reasoning {
            Some(enabled) => wire.serialize_field("reasoning", &WireReasoning { enabled })?,
            None => wire.skip_field("reasoning")?,
        }
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

/// The reasoning control on the wire.
///
/// @planks("the request disables reasoning")
#[derive(serde::Serialize)]
struct WireReasoning {
    enabled: bool,
}

/// The acronym request: the bundled skill's name and description, sampled at a
/// very high temperature.
///
/// @planks("the acronym request is built")
/// @planks("the acronym request and the assistant request are built")
/// @planks("the request disables reasoning")
pub fn acronym_request(settings: &Settings) -> Request {
    let mut request = request(
        settings,
        ACRONYM_TEMPERATURE,
        crate::skill::acronym_context(),
    );
    // A reasoning model spends its thinking budget on whatever it is asked, and
    // the tagline asks for six words, so the request turns reasoning off rather
    // than the ceiling growing to fit it.
    request.reasoning = Some(false);
    request
}

/// The assistant request: the whole bundled skill body and the operator's
/// question, sampled at a low temperature, with no earlier exchange ahead of
/// it.
///
/// @planks("the assistant request is built")
/// @planks("the acronym request and the assistant request are built")
pub fn assistant_request(settings: &Settings, question: &str) -> Request {
    assistant_turn_request(settings, &[], question)
}

/// The assistant request for `question`, carrying the session's compacted
/// transcript ahead of it. The seventeen most recent exchanges are carried
/// whole; older ones keep their question and lose their answer; and when the
/// transcript still exceeds the character budget after that, the oldest
/// question goes too, so forgetting is the last resort rather than the
/// mechanism. The bundled skill body is attached once, to the earliest
/// message the compacted transcript still carries.
///
/// @planks("the assistant request carries no earlier exchange")
/// @planks("the assistant request carries the earlier question {string}")
/// @planks("the assistant request carries the earlier answer {string}")
/// @planks("the assistant request carries the question {string}")
/// @planks("the assistant request carries no answer for {string}")
/// @planks("the assistant request carries seventeen whole exchanges")
pub fn assistant_turn_request(
    settings: &Settings,
    history: &[Exchange],
    question: &str,
) -> Request {
    let mut messages: Vec<(String, String)> = Vec::new();
    for exchange in compact(history) {
        let content = if messages.is_empty() {
            format!(
                "{}\n{}",
                crate::skill::assistant_context(),
                exchange.question
            )
        } else {
            exchange.question
        };
        messages.push(("user".to_string(), content));
        if let Some(answer) = exchange.answer {
            messages.push(("assistant".to_string(), answer));
        }
    }
    let content = if messages.is_empty() {
        format!("{}\n{question}", crate::skill::assistant_context())
    } else {
        question.to_string()
    };
    messages.push(("user".to_string(), content));
    request_from_messages(settings, ASSISTANT_TEMPERATURE, messages)
}

/// `history`, with the seventeen most recent exchanges kept whole, older ones
/// stripped to their question, and the oldest question dropped in turn while
/// the remaining transcript still exceeds the character budget.
fn compact(history: &[Exchange]) -> Vec<Exchange> {
    let boundary = history.len().saturating_sub(WHOLE_WINDOW);
    let mut compacted: Vec<Exchange> = history
        .iter()
        .enumerate()
        .map(|(index, exchange)| Exchange {
            question: exchange.question.clone(),
            answer: if index < boundary {
                None
            } else {
                exchange.answer.clone()
            },
        })
        .collect();
    while transcript_len(&compacted) > TRANSCRIPT_BUDGET && !compacted.is_empty() {
        compacted.remove(0);
    }
    compacted
}

/// The character count `exchanges` spends, the quantity the transcript budget
/// bounds.
fn transcript_len(exchanges: &[Exchange]) -> usize {
    exchanges
        .iter()
        .map(|exchange| exchange.question.len() + exchange.answer.as_deref().map_or(0, str::len))
        .sum()
}

/// Build a single-message request against the configured settings. The
/// credential becomes a bearer token, absent when no credential is
/// configured.
///
/// @planks("the request carries the authorization header {string}")
/// @planks("the request carries no authorization header")
fn request(settings: &Settings, temperature: f64, prompt: String) -> Request {
    request_from_messages(settings, temperature, vec![("user".to_string(), prompt)])
}

/// Build a request carrying `messages` against the configured settings. The
/// credential becomes a bearer token, absent when no credential is
/// configured.
fn request_from_messages(
    settings: &Settings,
    temperature: f64,
    messages: Vec<(String, String)>,
) -> Request {
    Request {
        base_url: settings.base_url.clone(),
        model: settings.model.clone(),
        authorization: settings.api_key.as_ref().map(|key| format!("Bearer {key}")),
        temperature,
        messages,
        reasoning: None,
    }
}

/// The acronym expansion an interactive help fills its tagline with, absent
/// when no credential is configured, when the provider cannot be reached,
/// when it rejects the credential, when it generates nothing, or when it has
/// answered nothing within the tagline ceiling.
///
/// @planks("the operator runs {string} in an interactive terminal")
/// @planks("the tagline is generated")
pub fn tagline_expansion(settings: &Settings) -> Option<String> {
    let deadline = std::time::Instant::now() + TAGLINE_CEILING;
    for _ in 0..ACRONYM_ASKS {
        let remaining = deadline.checked_duration_since(std::time::Instant::now())?;
        let generated = generated_expansion(settings, remaining)?;
        if let Some(raised) = raised(&generated) {
            return Some(raised);
        }
    }
    None
}

/// The tagline generation in flight, started before the help is drawn so the
/// model works while the operator reads the rest. The ceiling runs from the
/// moment the generation starts rather than from the moment the help asks for
/// it, so the wait the help inherits is the ceiling less whatever the drawing
/// took.
///
/// @planks("the operator runs {string} in an interactive terminal")
pub fn pending_tagline(settings: &Settings) -> PendingTagline {
    let deadline = std::time::Instant::now() + TAGLINE_CEILING;
    let settings = settings.clone();
    let (generated, expansion) = std::sync::mpsc::channel();
    std::thread::spawn(move || generated.send(tagline_expansion(&settings)));
    PendingTagline {
        deadline,
        expansion,
    }
}

/// A tagline generation the help is waiting on.
pub struct PendingTagline {
    /// The instant the wait ends, whatever the provider is doing by then.
    deadline: std::time::Instant,
    /// The expansion the generation produced, absent where it produced none.
    expansion: std::sync::mpsc::Receiver<Option<String>>,
}

impl PendingTagline {
    /// The expansion, waited for no longer than the ceiling leaves. A provider
    /// that has not answered by then is read as no answer, so the line settles
    /// on the unavailable notice inside the ceiling rather than on whatever the
    /// call takes to give up.
    ///
    /// @planks("the operator runs {string} in an interactive terminal")
    pub fn settled(self) -> Option<String> {
        let remaining = self
            .deadline
            .saturating_duration_since(std::time::Instant::now());
        self.expansion.recv_timeout(remaining).ok().flatten()
    }
}

/// `expansion` with the letters that spell the name raised, absent when its
/// words do not begin with those letters in order. A model claims its words
/// spell the name; this walks the expansion and checks the claim, and raising
/// the letters shows the reader where they are.
fn raised(expansion: &str) -> Option<String> {
    let characters: Vec<char> = expansion.chars().collect();
    let mut letters = ACRONYM_LETTERS.chars();
    let mut wanted = letters.next();
    let mut raised = String::with_capacity(expansion.len());
    for (index, character) in characters.iter().enumerate() {
        let begins_word =
            character.is_alphabetic() && (index == 0 || !characters[index - 1].is_alphanumeric());
        if begins_word && wanted == Some(character.to_ascii_lowercase()) {
            raised.extend(character.to_uppercase());
            wanted = letters.next();
        } else {
            raised.push(*character);
        }
    }
    wanted.is_none().then_some(raised)
}

/// The acronym expansion the configured provider generated within `ceiling`,
/// absent when no credential is configured, when the provider cannot be
/// reached, when it rejects the credential, or when it generates nothing.
fn generated_expansion(settings: &Settings, ceiling: std::time::Duration) -> Option<String> {
    settings.api_key.as_ref()?;
    let generated = complete(&acronym_request(settings), ceiling)?;
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
    assistant_completion_for_turn(settings, &[], question)
}

/// `assistant_completion`, carrying `history`'s compacted transcript ahead of
/// `question`.
///
/// @planks("the operator asks {string}")
pub fn assistant_completion_for_turn(
    settings: &Settings,
    history: &[Exchange],
    question: &str,
) -> Option<String> {
    complete(
        &assistant_turn_request(settings, history, question),
        GENERATION_CEILING,
    )
}

/// The terminal object model the configured provider read from the screen it
/// was shown, absent when no credential is configured, when the provider cannot
/// be reached, when it rejects the credential, or when it answers with nothing.
///
/// @planks("the terminal object model is inferred")
/// @planks("the terminal object model is inferred by the configured engine")
pub fn tom_completion(settings: &Settings, screen: &str) -> Option<String> {
    settings.api_key.as_ref()?;
    let prompt = format!("{TOM_INSTRUCTION}{screen}");
    complete(&tom_reading(settings, prompt), GENERATION_CEILING)
}

/// The reading request a naming pass sends, carrying `prompt`. A reasoning model
/// spends its thinking budget on whatever it is asked, and reading a screen into
/// a document is asked of every inspection, so the request turns reasoning off
/// rather than the ceiling growing to fit it.
///
/// @planks("the operator inspects the fixture terminal program")
fn tom_reading(settings: &Settings, prompt: String) -> Request {
    let mut request = request(settings, TOM_TEMPERATURE, prompt);
    request.reasoning = Some(false);
    request
}

/// The request a naming pass over `program` sends: the reading instruction, the
/// program's tldr page where the configured source carries one, and the screen.
/// A program the source has no page for is named from the screen alone, which is
/// the reading every other pass already makes.
///
/// @planks("the terminal object model of {string} is inferred")
/// @planks("the inference request carries the tldr page for {string}")
/// @planks("the inference request carries no tldr page")
pub fn tom_request(settings: &Settings, screen: &str, program: &str) -> Request {
    let prompt = match crate::examples::fetch_page(&settings.tldr_base_url, program) {
        Some(page) => format!("{TOM_INSTRUCTION}{TOM_PAGE_INSTRUCTION}{page}\n{screen}"),
        None => format!("{TOM_INSTRUCTION}{screen}"),
    };
    tom_reading(settings, prompt)
}

/// `tom_completion`, naming the program the screen belongs to so the pass reads
/// its page.
///
/// @planks("the terminal object model of {string} is inferred")
/// @planks("the plan for {string} is written")
pub fn tom_completion_for(settings: &Settings, screen: &str, program: &str) -> Option<String> {
    settings.api_key.as_ref()?;
    complete(&tom_request(settings, screen, program), GENERATION_CEILING)
}

/// Whether the configured provider answers, bounded by the probe ceiling. The
/// ceiling is enforced from the outside on a background thread rather than
/// trusted to the HTTP client's own internal timeout, which the client can
/// overrun: the wait ends the moment the probe ceiling elapses, whatever the
/// call beneath it is still doing.
///
/// @planks("Tinman checks whether inference is available")
pub fn is_available(settings: &Settings) -> bool {
    is_available_within(settings, PROBE_CEILING)
}

/// Whether the configured provider answers within `ceiling`. The ceiling is
/// enforced from the outside on a background thread rather than trusted to the
/// HTTP client's own internal timeout, which the client can overrun: the wait
/// ends the moment the ceiling elapses, whatever the call beneath it is still
/// doing.
///
/// @planks("that seam is exercised")
pub fn is_available_within(settings: &Settings, ceiling: std::time::Duration) -> bool {
    let settings = settings.clone();
    let (answered, available) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = answered.send(generated_expansion(&settings, ceiling).is_some());
    });
    available.recv_timeout(ceiling).unwrap_or(false)
}

/// Send a request to the configured provider and return the content it
/// generated. Absent when the provider cannot be reached, rejects the request,
/// or answers with nothing.
///
/// @planks("the acronym request is sent to that endpoint")
fn complete(request: &Request, ceiling: std::time::Duration) -> Option<String> {
    let url = request.url();
    let wire = serde_json::to_value(request).ok()?;
    let body = wire.get("body")?;
    let body = serde_json::to_string(body).ok()?;
    let mut call = ureq::post(&url)
        .config()
        .timeout_global(Some(ceiling))
        .build()
        .header("Content-Type", "application/json");
    if let Some(authorization) = &request.authorization {
        call = call.header("Authorization", authorization);
    }
    let mut response = call.send(&body).ok()?;
    let text = response.body_mut().read_to_string().ok()?;
    let completion: Completion = serde_json::from_str(&text).ok()?;
    completion
        .choices
        .into_iter()
        .next()
        .map(|choice| choice.message.content)
}

/// The provider's reply, decoded with the JSON library already in the
/// dependency graph.
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

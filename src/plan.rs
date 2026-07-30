//! The harness plan: the canonical YAML representation of a recorded flow, and
//! the shorthands that normalize into it. A plan is either the full form,
//! carrying a flow of process entries, or the shorthand form, carrying one
//! terminal command and its steps. Both parse to the same plan.

use crate::sandbox::SandboxSpec;

/// A parsed harness plan: the sandbox the flow runs in, the ordered flow, and
/// what the plan was written from. A plan read from nothing outside the binary
/// carries no sources, and none are written back.
///
/// @planks("the two parsed plans are identical")
/// @planks("it conforms to the {string} schema in {string}")
/// @planks("a recorded plan is serialized and read back")
#[derive(Debug, Clone, serde::Serialize)]
pub struct Plan {
    pub sandbox: SandboxSpec,
    #[serde(with = "serde_yaml::with::singleton_map_recursive")]
    pub flow: Vec<FlowStep>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<Source>,
}

/// One thing a plan was written from: the source read, and the terms that came
/// with it. A source whose terms were not stated carries no licence.
///
/// @planks("a recorded plan is serialized and read back")
/// @planks("the verifier checks the plan deserialization strictness contract")
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub licence: Option<String>,
}

/// One process in the flow: a plain command, or a terminal program driven by
/// steps. Written as a single-keyword map, so the flow entry names the process
/// kind it carries.
///
/// @planks("the plan's flow holds {int} step")
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FlowStep {
    Run(RunProcess),
    Tui(TuiProcess),
}

/// A command and how the plan expects it to run: the directory it runs in, the
/// exit status the command must leave, and the text fed to its standard input.
///
/// @planks("the flow passes")
/// @planks("execution fails and reports the status {int}")
/// @planks("the step's standard output is {string}")
/// @planks("a flow whose only step runs {string} in the directory {string}")
/// @planks("the verifier checks the plan deserialization strictness contract")
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(from = "RunForm", deny_unknown_fields)]
pub struct RunProcess {
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    pub status: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdin: Option<String>,
}

/// The written forms of a run entry: a bare command, or a command carrying the
/// directory, the status, and the standard input the plan expects. A bare
/// command YAML reads as a boolean, such as the shell's `false`, is that
/// command's name.
///
/// @planks("the flow passes")
/// @planks("execution fails and reports the step that failed")
/// @planks("a flow whose only step runs {string} in the directory {string}")
/// @planks("parsing fails and reports the unknown field {string}")
enum RunForm {
    Command(String),
    Boolean(bool),
    Full(RunFull),
}

/// The map form of a run entry: the command, the directory it runs in, the exit
/// status expected, and the standard input fed to it.
///
/// @planks("a flow whose only step runs {string} in the directory {string}")
/// @planks("parsing fails and reports the unknown field {string}")
/// @planks("the verifier checks the plan deserialization strictness contract")
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct RunFull {
    command: String,
    #[serde(default)]
    cwd: Option<String>,
    #[serde(default)]
    status: i32,
    #[serde(default)]
    stdin: Option<String>,
}

impl<'de> serde::Deserialize<'de> for RunForm {
    /// Read a run entry by the shape it was written in. The written shape
    /// selects the form, so the map form's own failure is the failure reported:
    /// a field the map form does not define is named rather than lost behind a
    /// report that no form matched.
    ///
    /// @planks("parsing fails and reports the unknown field {string}")
    fn deserialize<D>(deserializer: D) -> Result<RunForm, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match serde_yaml::Value::deserialize(deserializer)? {
            serde_yaml::Value::String(command) => Ok(RunForm::Command(command)),
            serde_yaml::Value::Bool(flag) => Ok(RunForm::Boolean(flag)),
            written => RunFull::deserialize(written)
                .map(RunForm::Full)
                .map_err(serde::de::Error::custom),
        }
    }
}

impl From<RunForm> for RunProcess {
    /// Normalize a written run entry: a bare command runs in the workspace
    /// itself, expects the status 0, and carries no input.
    ///
    /// @planks("the flow passes")
    /// @planks("the step's standard output is {string}")
    /// @planks("a flow whose only step runs {string} in the directory {string}")
    fn from(form: RunForm) -> RunProcess {
        match form {
            RunForm::Command(command) => RunProcess {
                command,
                cwd: None,
                status: 0,
                stdin: None,
            },
            RunForm::Boolean(flag) => RunProcess {
                command: flag.to_string(),
                cwd: None,
                status: 0,
                stdin: None,
            },
            RunForm::Full(full) => RunProcess {
                command: full.command,
                cwd: full.cwd,
                status: full.status,
                stdin: full.stdin,
            },
        }
    }
}

/// A terminal program and the steps driving it.
///
/// @planks("the flow's first step drives the command {string}")
/// @planks("the verifier checks the plan deserialization strictness contract")
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TuiProcess {
    pub command: String,
    #[serde(default, with = "serde_yaml::with::singleton_map_recursive")]
    pub steps: Vec<Action>,
}

/// One semantic action against the driven program. Written as a single-keyword
/// map, so an unrecognized keyword is reported as an unknown variant.
///
/// @planks("parsing fails and reports the unknown step keyword {string}")
/// @planks("the written plan records a key press {string}")
/// @planks("a recorded plan is serialized and read back")
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    Activate(Locator),
    Fill(Fill),
    Press(String),
    Expect(Expectation),
    Capture(Capture),
}

/// Collect the items of a pane into named structured data: the role the pane
/// plays, the role its items play, how much of the pane is read, and the name
/// the collected items are kept under.
///
/// @planks("a recorded plan is serialized and read back")
/// @planks("the verifier checks the plan deserialization strictness contract")
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Capture {
    pub role: String,
    pub items: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(rename = "as")]
    pub name: String,
}

/// Addresses a region of the terminal object model by name, and by role when
/// the plan names one. A locator a recording confirmed also carries the region
/// it was scoped to and the binding it needed.
///
/// @planks("the step's locator name is {string}")
/// @planks("the step's locator names no role")
/// @planks("the plan records the locator's binding as {string}")
/// @planks("a recorded plan is serialized and read back")
/// @planks("that plan is replayed")
/// @planks("the verifier checks the plan deserialization strictness contract")
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(from = "LocatorForm", deny_unknown_fields)]
pub struct Locator {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub within: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binding: Option<String>,
}

/// The two written forms of a locator: a bare name, or a role and a name with
/// the scope and binding a confirmed locator records.
///
/// @planks("the step's locator names no role")
/// @planks("a recorded plan is serialized and read back")
/// @planks("that plan is replayed")
/// @planks("the verifier checks the plan deserialization strictness contract")
#[derive(serde::Deserialize)]
#[serde(untagged, deny_unknown_fields)]
enum LocatorForm {
    Name(String),
    Full {
        role: Option<String>,
        name: String,
        #[serde(default)]
        within: Option<String>,
        #[serde(default)]
        binding: Option<String>,
    },
}

impl From<LocatorForm> for Locator {
    /// Normalize a written locator: a bare name carries no role.
    ///
    /// @planks("the step's locator name is {string}")
    /// @planks("the step's locator names no role")
    /// @planks("a recorded plan is serialized and read back")
    /// @planks("that plan is replayed")
    fn from(form: LocatorForm) -> Locator {
        match form {
            LocatorForm::Name(name) => Locator {
                role: None,
                name,
                within: None,
                binding: None,
            },
            LocatorForm::Full {
                role,
                name,
                within,
                binding,
            } => Locator {
                role,
                name,
                within,
                binding,
            },
        }
    }
}

/// Enter a value into the textbox carrying a label.
///
/// @planks("the two parsed plans are identical")
/// @planks("the verifier checks the plan deserialization strictness contract")
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fill {
    pub label: String,
    pub value: String,
}

/// Assert text is present on screen, in the region a recorded expectation
/// addresses. A hand-authored expectation names that region by name under
/// `within`, so the step stays bound to the region rather than to the cells the
/// region occupied on the terminal it was captured on.
///
/// @planks("the step expects the text {string}")
/// @planks("the plan records the locator's binding as {string}")
/// @planks("a harness plan whose step expects the status bar to contain {string}, captured at {int} columns")
/// @planks("the verifier checks the plan deserialization strictness contract")
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(from = "ExpectationForm", deny_unknown_fields)]
pub struct Expectation {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub within: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locator: Option<Locator>,
}

/// The two written forms of an expectation: a bare string, or a text map
/// carrying the region the expectation is scoped to and the locator a recorded
/// expectation addresses.
///
/// @planks("the step expects the text {string}")
/// @planks("a harness plan whose step expects the status bar to contain {string}, captured at {int} columns")
/// @planks("the verifier checks the plan deserialization strictness contract")
#[derive(serde::Deserialize)]
#[serde(untagged, deny_unknown_fields)]
enum ExpectationForm {
    Text(String),
    Full {
        text: String,
        #[serde(default)]
        within: Option<String>,
        #[serde(default)]
        locator: Option<Locator>,
    },
}

impl From<ExpectationForm> for Expectation {
    /// Normalize a written expectation: a bare string is the expected text.
    ///
    /// @planks("the step expects the text {string}")
    /// @planks("a harness plan whose step expects the status bar to contain {string}, captured at {int} columns")
    fn from(form: ExpectationForm) -> Expectation {
        match form {
            ExpectationForm::Text(text) => Expectation {
                text,
                within: None,
                locator: None,
            },
            ExpectationForm::Full {
                text,
                within,
                locator,
            } => Expectation {
                text,
                within,
                locator,
            },
        }
    }
}

/// The plan as written: the full form's flow, or the shorthand form's command
/// and steps, with the sandbox section optional in both.
///
/// @planks("parsing fails and reports a missing flow")
/// @planks("a recorded plan is serialized and read back")
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PlanForm {
    sandbox: Option<SandboxSpec>,
    #[serde(default, with = "serde_yaml::with::singleton_map_recursive")]
    flow: Option<Vec<FlowStep>>,
    tui: Option<String>,
    #[serde(default, with = "serde_yaml::with::singleton_map_recursive")]
    steps: Vec<Action>,
    #[serde(default)]
    sources: Vec<Source>,
}

/// Restate a deserializer's type complaint as the operator's own mistake. A
/// plan's values are text a program printed or an operator reads on a screen,
/// so a bare 1.0 is a value YAML read as a number, and the fix is a pair of
/// quotes. A complaint of any other shape passes through as written.
///
/// @planks("parsing fails and reports the value must be quoted to be read as text")
fn name_the_quoting_fix(message: &str) -> String {
    let Some((path, complaint)) = message.split_once("invalid type: ") else {
        return message.to_string();
    };
    let Some((found, location)) = complaint.split_once(", expected a string") else {
        return message.to_string();
    };
    let Some(value) = found.split('`').nth(1) else {
        return message.to_string();
    };
    format!("{path}{value} must be quoted as \"{value}\" to be read as text{location}")
}

/// Parse a harness plan, normalizing every shorthand into the canonical plan.
/// An omitted sandbox section is the secure default.
///
/// @planks("the plan is parsed")
/// @planks("both plans are parsed")
/// @planks("the harness plan at {string}")
/// @planks("parsing fails and reports a missing flow")
/// @planks("parsing fails and reports the value must be quoted to be read as text")
/// @planks("a recorded plan is serialized and read back")
pub fn parse(source: &str) -> Result<Plan, String> {
    let form: PlanForm =
        serde_yaml::from_str(source).map_err(|e| name_the_quoting_fix(&e.to_string()))?;
    let flow = if let Some(command) = form.tui {
        vec![FlowStep::Tui(TuiProcess {
            command,
            steps: form.steps,
        })]
    } else {
        form.flow
            .ok_or_else(|| "the plan defines no flow".to_string())?
    };
    Ok(Plan {
        sandbox: form.sandbox.unwrap_or_default(),
        flow,
        sources: form.sources,
    })
}

/// Parse a plan's sandbox section on its own, so a section can be read without
/// a surrounding plan.
///
/// @planks("the sandbox specification is parsed")
pub fn parse_sandbox(source: &str) -> Result<SandboxSpec, String> {
    serde_yaml::from_str(source).map_err(|e| e.to_string())
}

//! The harness plan: the canonical YAML representation of a recorded flow, and
//! the shorthands that normalize into it. A plan is either the full form,
//! carrying a flow of process entries, or the shorthand form, carrying one
//! terminal command and its steps. Both parse to the same plan.

use crate::sandbox::SandboxSpec;

/// A parsed harness plan: the sandbox the flow runs in, and the ordered flow.
///
/// @planks("the two parsed plans are identical")
/// @planks("it conforms to the {string} schema in {string}")
#[derive(Debug, Clone, serde::Serialize)]
pub struct Plan {
    pub sandbox: SandboxSpec,
    #[serde(with = "serde_yaml::with::singleton_map_recursive")]
    pub flow: Vec<FlowStep>,
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

/// A command and how the plan expects it to run: the exit status the command
/// must leave, and the text fed to its standard input.
///
/// @planks("the flow passes")
/// @planks("execution fails and reports the status {int}")
/// @planks("the step's standard output is {string}")
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(from = "RunForm")]
pub struct RunProcess {
    pub command: String,
    pub status: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdin: Option<String>,
}

/// The written forms of a run entry: a bare command, or a command carrying the
/// status and the standard input the plan expects. A bare command YAML reads as
/// a boolean, such as the shell's `false`, is that command's name.
///
/// @planks("the flow passes")
/// @planks("execution fails and reports the step that failed")
#[derive(serde::Deserialize)]
#[serde(untagged)]
enum RunForm {
    Command(String),
    Boolean(bool),
    Full {
        command: String,
        #[serde(default)]
        status: i32,
        #[serde(default)]
        stdin: Option<String>,
    },
}

impl From<RunForm> for RunProcess {
    /// Normalize a written run entry: a bare command expects the status 0 and
    /// carries no input.
    ///
    /// @planks("the flow passes")
    /// @planks("the step's standard output is {string}")
    fn from(form: RunForm) -> RunProcess {
        match form {
            RunForm::Command(command) => RunProcess {
                command,
                status: 0,
                stdin: None,
            },
            RunForm::Boolean(flag) => RunProcess {
                command: flag.to_string(),
                status: 0,
                stdin: None,
            },
            RunForm::Full {
                command,
                status,
                stdin,
            } => RunProcess {
                command,
                status,
                stdin,
            },
        }
    }
}

/// A terminal program and the steps driving it.
///
/// @planks("the flow's first step drives the command {string}")
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    Activate(Locator),
    Fill(Fill),
    Press(String),
    Expect(Expectation),
}

/// Addresses a region of the terminal object model by name, and by role when
/// the plan names one. A locator a recording confirmed also carries the region
/// it was scoped to and the binding it needed.
///
/// @planks("the step's locator name is {string}")
/// @planks("the step's locator names no role")
/// @planks("the plan records the locator's binding as {string}")
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(from = "LocatorForm")]
pub struct Locator {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binding: Option<String>,
}

/// The two written forms of a locator: a bare name, or a role and a name with
/// the scope and binding a confirmed locator records.
///
/// @planks("the step's locator names no role")
#[derive(serde::Deserialize)]
#[serde(untagged)]
enum LocatorForm {
    Name(String),
    Full {
        role: Option<String>,
        name: String,
        #[serde(default)]
        scope: Option<String>,
        #[serde(default)]
        binding: Option<String>,
    },
}

impl From<LocatorForm> for Locator {
    /// Normalize a written locator: a bare name carries no role.
    ///
    /// @planks("the step's locator name is {string}")
    /// @planks("the step's locator names no role")
    fn from(form: LocatorForm) -> Locator {
        match form {
            LocatorForm::Name(name) => Locator {
                role: None,
                name,
                scope: None,
                binding: None,
            },
            LocatorForm::Full {
                role,
                name,
                scope,
                binding,
            } => Locator {
                role,
                name,
                scope,
                binding,
            },
        }
    }
}

/// Enter a value into the textbox carrying a label.
///
/// @planks("the two parsed plans are identical")
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(from = "ExpectationForm")]
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
#[derive(serde::Deserialize)]
#[serde(untagged)]
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
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PlanForm {
    sandbox: Option<SandboxSpec>,
    #[serde(default, with = "serde_yaml::with::singleton_map_recursive")]
    flow: Option<Vec<FlowStep>>,
    tui: Option<String>,
    #[serde(default, with = "serde_yaml::with::singleton_map_recursive")]
    steps: Vec<Action>,
}

/// Parse a harness plan, normalizing every shorthand into the canonical plan.
/// An omitted sandbox section is the secure default.
///
/// @planks("the plan is parsed")
/// @planks("both plans are parsed")
/// @planks("the harness plan at {string}")
/// @planks("parsing fails and reports a missing flow")
pub fn parse(source: &str) -> Result<Plan, String> {
    let form: PlanForm = serde_yaml::from_str(source).map_err(|e| e.to_string())?;
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
    })
}

/// Parse a plan's sandbox section on its own, so a section can be read without
/// a surrounding plan.
///
/// @planks("the sandbox specification is parsed")
pub fn parse_sandbox(source: &str) -> Result<SandboxSpec, String> {
    serde_yaml::from_str(source).map_err(|e| e.to_string())
}

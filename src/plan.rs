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
    Run(String),
    Tui(TuiProcess),
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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    Activate(Locator),
    Fill(Fill),
    Expect(Expectation),
}

/// Addresses a region of the terminal object model by name, and by role when
/// the plan names one.
///
/// @planks("the step's locator name is {string}")
/// @planks("the step's locator names no role")
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(from = "LocatorForm")]
pub struct Locator {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    pub name: String,
}

/// The two written forms of a locator: a bare name, or a role and a name.
///
/// @planks("the step's locator names no role")
#[derive(serde::Deserialize)]
#[serde(untagged)]
enum LocatorForm {
    Name(String),
    Full { role: Option<String>, name: String },
}

impl From<LocatorForm> for Locator {
    /// Normalize a written locator: a bare name carries no role.
    ///
    /// @planks("the step's locator name is {string}")
    /// @planks("the step's locator names no role")
    fn from(form: LocatorForm) -> Locator {
        match form {
            LocatorForm::Name(name) => Locator { role: None, name },
            LocatorForm::Full { role, name } => Locator { role, name },
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

/// Assert text is present on screen.
///
/// @planks("the step expects the text {string}")
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(from = "ExpectationForm")]
pub struct Expectation {
    pub text: String,
}

/// The two written forms of an expectation: a bare string, or a text map.
///
/// @planks("the step expects the text {string}")
#[derive(serde::Deserialize)]
#[serde(untagged)]
enum ExpectationForm {
    Text(String),
    Full { text: String },
}

impl From<ExpectationForm> for Expectation {
    /// Normalize a written expectation: a bare string is the expected text.
    ///
    /// @planks("the step expects the text {string}")
    fn from(form: ExpectationForm) -> Expectation {
        match form {
            ExpectationForm::Text(text) => Expectation { text },
            ExpectationForm::Full { text } => Expectation { text },
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

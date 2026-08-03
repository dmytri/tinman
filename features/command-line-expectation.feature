Feature: command-line expectation
  As an operator exploring a terminal program
  I want to state one expectation from the command line
  So that I learn what a program exposes without writing a plan first

  Rule: the command is named for the plan step it spells, because a plan step, a driver method and a command line that mean one thing are one concept and carry one name. Three names for an expectation would leave a reader of any two of them guessing whether the third does something else, and the surface that drifts first is the one nobody is testing against.

  Rule: this command is a probe rather than a test, and the difference decides what it produces. A live screen changes under it, so an answer it prints is true of one moment and is believed or not. What survives is the plan it writes, which the operator commits and re-runs and watches go red the day the program changes. An explainer hands over a verdict; a probe hands over evidence. Playwright reaches the same artifact from the other direction, recording a session into a test file rather than reporting on one.

  Rule: an expectation names either the text it wants on the screen or the region it wants to resolve, and both forms are the ones a plan already carries. The text form is the plan's bare expectation. The locator form is the plan's own locator, a role and a name, so no third vocabulary is coined for the command line. A region's name is its content in this model, so a locator naming both role and name is the whole of what stating a value means here.

  @sandbox
  Scenario: a text expectation that holds exits zero
    When the operator executes "tinman expect PID top"
    Then the command exits with a zero status

  @sandbox
  Scenario: a locator expectation resolves by role and name
    When the operator executes "tinman expect --role columnheader --name PID top"
    Then the command exits with a zero status

  Rule: a failed expectation has to show the screen it read, because the operator's next act is to correct it and the only thing that tells them how is what the program actually drew. A bare non-zero status sends them back to inspect to ask a question the failing command already knew the answer to.

  Rule: a failure reaches a person and an agent, and neither reads Rust. A locator that found nothing reported the region it wanted as `Some("NOSUCH")`, which is the language's own debug rendering of an optional value leaking through a message written for an operator. It names the thing the reader asked for wrapped in a type they have no reason to know exists.

  Rule: what a run can decide here is narrower than the fault, and saying so is better than implying otherwise. A message carrying the wrapper syntax of an optional or a result is mechanically findable and is checked; a message that is merely unclear is not, and no check will catch one.

  @sandbox
  Scenario: a failed locator names the region without Rust wrapper syntax
    When the operator executes "tinman expect --role columnheader --name NOSUCH top"
    Then the command exits with a non-zero status
    And the failure names "NOSUCH" as the region it looked for
    And the failure carries no optional or result wrapper syntax

  @sandbox
  Scenario: a failed expectation reports the screen it found
    When the operator executes "tinman expect NOSUCH top"
    Then the command exits with a non-zero status
    And the failure carries the screen the program drew

  Rule: a locator that resolves to nothing is the false-green shape this project keeps paying for, arriving at a new surface. A focused run naming a scenario that does not exist selects zero and exits green; a glob matching no file lints nothing and exits green. An expectation whose locator matches no region has asserted nothing, so it fails, and a mistyped role is reported as one rather than rewarded with a pass.

  @sandbox
  Scenario: a locator naming a role the model does not produce fails
    When the operator executes "tinman expect --role statusbar --name PID top"
    Then the command exits with a non-zero status
    And the failure reports that "statusbar" is not a role the model produces

  Rule: resolution answers what a locator matches and reports ambiguity as ambiguity, which is the standing rule for every locator this project resolves. Picking the first of several matches would answer a question the operator did not ask, and the answer would be stable enough to trust and wrong the moment a row moved.

  Rule: the same message is built on two paths and only one was corrected, so the wrapper leak survived where no assertion reached it. This scenario exercised the ambiguity arm while asserting only the count, which is why it stayed green through the fix beside it. A message assembled twice needs the assertion on both halves, or the half nobody asserts is the half that rots.

  Rule: a locator carrying no name has no name to report, so a message naming one describes a locator the operator did not write. Rendering the absent name as the language's word for absence is the wrapper leak and the wrong sentence at once.

  @sandbox
  Scenario: a locator matching several regions reports the ambiguity
    When the operator executes "tinman expect --role cell top"
    Then the command exits with a non-zero status
    And the failure reports the locator matched more than one region
    And the failure carries no optional or result wrapper syntax
    And the failure reports no name for a locator that carries none

  Rule: only an honoured probe earns an expectation. A plan written from a probe that failed hands the operator a permanently red test describing a screen that never held. This is the documented-examples rule reaching a second surface, and it is the whole reason the command writes a file at all. The option is spelled the way `record` and `inspect` already spell it, because commands that write a plan should not spell it three ways.

  @sandbox
  Scenario: a held expectation writes the plan it proved
    When the operator executes "tinman expect --role columnheader --name PID top --output probe.yaml"
    Then the command exits with a zero status
    And "probe.yaml" is a plan whose expectation names the role "columnheader" and the name "PID"
    And that expectation carries no text

  @sandbox
  Scenario: a failed expectation writes no plan
    When the operator executes "tinman expect NOSUCH top --output probe.yaml"
    Then the command exits with a non-zero status
    And no file is written at "probe.yaml"

  Rule: a command that launches a program is a launch path, and this project has shipped a launch path that ran unsandboxed while a scenario reported it isolated. The target reaches for something only the operator's real machine carries and the assertion is that it did not find it, so the scenario fails if the sandbox is ever bypassed rather than passing on a screen that never tried.

  @sandbox
  Scenario: expect launches its target inside the sandbox
    Given the operator's environment carries "TINMAN_SECRET"
    When the operator states an expectation against a target that prints whether it read "TINMAN_SECRET"
    Then the target reports the variable was absent

  Rule: the plan gains the locator form of an expectation in the same pass as the command that writes it, because a command writing a plan the schema rejects is a probe whose evidence nobody can commit. The text form is untouched, so every plan already written stays valid.

  Rule: reaching a state richer than a keypress is a plan's job, so the command line borrows the plan rather than growing verbs for activating, filling and capturing. A plan already expresses any sequence, is already committed, and is already replayable, so borrowing it costs no new vocabulary and leaves one artifact describing how a state is reached.

  Rule: the expectation names something only the plan's steps can produce, which is what makes this scenario able to fail. An expectation that holds on the program's opening screen would pass whether the plan ran or not, so it would report the option working while the option did nothing, and no separate step asserting that the steps ran can rescue it.

  @sandbox
  Scenario: an expectation runs a plan's steps before asserting
    Given a plan at "reach.yaml" whose steps press "q" against the fixture terminal program
    When the operator expects "Quit?" after "reach.yaml" against the fixture terminal program
    Then the command exits with a zero status

  Rule: a plan that failed on its way to the state leaves the expectation asserting against a screen nobody asked for, and reporting that expectation as held would be the false-green shape wearing a new coat. The plan's own failure is the answer, and it is reported as the plan's rather than as the expectation's, so the operator fixes the step that actually broke.

  @sandbox
  Scenario: an expectation whose plan failed reports the plan's failure
    Given a plan at "reach.yaml" whose steps expect "NOSUCH" against "top"
    When the operator executes "tinman expect --role columnheader --name %CPU --after reach.yaml top"
    Then the command exits with a non-zero status
    And the failure reports the plan step that failed
    And the failure does not report the expectation as read

  @contract
  Scenario: a plan carrying the locator form of an expectation validates
    Given the plan fixture carrying an expectation naming a role and a name
    When the plan is validated
    Then it conforms to the "harness-plan" schema in "scantlings/harness-plan.schema.json"

  Rule: the schema and the parser are two readings of one plan language, and a form only one of them accepts reaches the operator as a plan that validates and will not run. This project has shipped that divergence in the other direction, as a schema promising a cwd the parser silently dropped, and the lesson is the same from either side: the two readings are joined by a scenario or they are joined by nobody. The locator form is the case in hand, since the probe writes it and the operator commits what the probe wrote.

  Scenario: a plan stating an expectation as a bare locator parses
    Given a harness plan whose step expects the "columnheader" named "%CPU" and states no text
    When the plan is parsed
    Then the plan's step expects the "columnheader" named "%CPU"
    And that step states no text

  Rule: a plan the command wrote is a plan the command can replay, and the round trip is where a serialization change is actually proved. A written form that parses in isolation still fails the operator if the runner it was written for refuses it, and the probe's whole promise is that what it hands over is evidence they can commit and re-run.

  @sandbox
  Scenario: the plan a locator expectation wrote replays
    Given a plan written by "tinman expect --role columnheader --name PID top --output probe.yaml"
    When the operator executes "tinman test probe.yaml"
    Then the command exits with a zero status

  Rule: dropping the duplicated text from the written plan removed the only thing replay had to fail on, so the round trip above went green while asserting nothing, and the plan the probe handed the operator to commit would have passed against a program it had never seen. An expectation awaiting an empty text is satisfied by every screen ever drawn. The rule for the command-line path is already stated above, that a locator matching no region has asserted nothing and so fails, and replay owes the same answer because it is the same expectation read by a different reader. The passing direction cannot show this; only the failing one can, which is why the two are specified together.

  @sandbox
  Scenario: a replayed locator expectation matching no region fails
    Given a plan at "probe.yaml" whose expectation names the "columnheader" named "NOSUCHREGION"
    When the operator executes "tinman test probe.yaml"
    Then the command exits with a non-zero status
    And the failure names "NOSUCHREGION" as the region it looked for

Feature: replay
  As a test author
  I want a recorded flow replayed exactly
  So that a captured interaction becomes a repeatable test

  Rule: replay is absolutely deterministic. It invokes no model and opens no network connection, whatever inference is configured.

  Rule: a plan captured on one operator's terminal runs on another's, and terminals differ in size. A rectangle fixed in cells cannot survive that crossing; a role and a name can, which is why a locator addresses a region rather than a position. The rectangle the model reports describes where a region was drawn on the run that produced it, and a plan that pins one has pinned a terminal size rather than a behaviour.

  Rule: terminal size is a property of the run and never of the plan. The caller supplies it, defaulting to the operator's own terminal, and it reaches the PTY and the virtual screen together. A plan that recorded its capture size would invite replay to restore that size, which is the one thing these scenarios exist to prevent.

  Scenario: a plan captured at one terminal width replays at another
    Given a harness plan captured from the fixture terminal program at 80 columns
    When that plan is replayed at 120 columns
    Then the replay passes

  Scenario: a status line stays bound when the terminal widens
    Given a harness plan whose step expects the status bar to contain "READY", captured at 80 columns
    When that plan is replayed at 120 columns
    Then the replay passes

  Scenario: a plan driving a program by activation and filling replays
    Given a plan whose steps activate a region and fill a field
    When the plan is replayed
    Then the plan passes

  Rule: a replay that satisfies its expectations has not thereby reproduced its interaction. A locator can bind a different region from the one recorded and pass anyway, where whatever it found happens to carry the expected text, and nothing in the result says so. That is the quiet-wrong-answer class this feature exists to prevent, arriving through the locator rather than through the rectangle, and it was invisible here for as long as this scenario's title said reproduction while its steps read only that the replay passed. Comparing whole screens would surface it and was declined on its own merits: asserting a screen is the same as last time is drift detection rather than specification, which is the ground on which this project already refused snapshot testing. What the plan records about each locator is narrower evidence that costs no snapshot, and the falsifiable part of it is the rung: role and name are what resolution already filters on, so comparing them against themselves can never fail, while the scope and ordinal a recording needed are discarded at replay and a plan that needed narrowing replays without it.

  Scenario: replaying a recorded flow reproduces the interaction
    Given a harness plan driving the fixture terminal program
    When that plan is replayed
    Then the replay passes

  Rule: the plan schema names the locator's narrowing key `within` and production declares it `scope`, so a plan carrying either loses it: the schema closes to additional properties and rejects `scope`, while production never reads `within`. Proven by validating both shapes against the schema. Nothing caught it because no check compares a scantling's declared properties against the fields production emits, which is the gap this pair of scenarios exists to close as much as the behaviour is.

  Rule: a plan captures the model at a point in its flow so a consumer can assert against what the screen held there, which is the capture limb of the tool's own job. The step form is declared in the plan schema and implemented in the plan module, and no scenario has ever exercised it, so it is unspecified behaviour rather than a broken promise: the round-trip check below fails on it for exactly that reason, a property nothing carries being indistinguishable from a property nothing supports.

  Scenario: a plan captures the model where its flow says to
    Given a harness plan whose flow captures the model after activating the "menuitem" named "Settings"
    When that plan is replayed
    Then the replay passes
    And the capture carries a "button" named "Save"

  @contract
  Scenario: every property the plan schema declares is a property the plan carries
    Given the properties declared by "scantlings/harness-plan.schema.json"
    When a recorded plan is serialized and read back
    Then every declared property survives the round trip
    And the properties read are not empty

  Scenario: a locator recorded with a scope replays inside that scope
    Given a harness plan whose step activates the "button" named "OK" within the region named "Settings"
    And the program under replay shows another "button" named "OK" outside that region
    When that plan is replayed
    Then the replay passes

  Scenario: a locator that binds a different region fails the replay
    Given a harness plan whose step activates the region named "Settings"
    And the program under replay has renamed that region
    When that plan is replayed
    Then the replay fails and reports the locator that bound no matching region

  Rule: a locator's role is optional in the plan language and in the plan schema, which records that an absent role matches any role. Three shapes the schema accepts reach a replay without one: an activation written as a bare name, an expectation written as a bare locator, and a scope naming a region the screen does not carry. A plan is operator input, so a shape the schema accepts is answered rather than fatal. The scenario above already fixes that a locator binding nothing reports the locator, and these carry the same contract to the shapes that currently abort the run instead, which leaves an operator a panic message and a status of 101 where every neighbouring failure leaves a sentence and a status of 1.

  Scenario: an activation written as a bare name is reported rather than fatal
    Given a harness plan whose step activates the bare name "Save"
    When that plan is replayed
    Then the replay fails and reports the locator that named no role

  Scenario: an expectation written as a bare locator is reported rather than fatal
    Given a harness plan whose step expects the region named "Save" and states no role
    When that plan is replayed
    Then the replay fails and reports the locator that named no role

  Scenario: a scope naming a region the screen does not carry is reported rather than fatal
    Given a harness plan whose step activates the "button" named "Save" within the region named "Nowhere"
    When that plan is replayed
    Then the replay fails and reports the scope that matched no region

  Scenario: a failed expectation names the step that failed
    Given a harness plan driving the fixture terminal program whose final step expects the text "Deployed"
    When that plan is replayed
    Then the replay fails and reports the step expecting "Deployed"

  Scenario: a failure report shows the screen the step saw
    Given a harness plan driving the fixture terminal program whose final step expects the text "Deployed"
    When that plan is replayed
    Then the failure report contains the text "Username"

  Scenario: replay performs no inference
    Given a harness plan driving the fixture terminal program
    And the inference credential is configured
    And the inference provider endpoint is unreachable
    When that plan is replayed
    Then the replay passes

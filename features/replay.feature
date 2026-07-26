Feature: replay
  As a test author
  I want a recorded flow replayed exactly
  So that a captured interaction becomes a repeatable test

  Rule: replay is absolutely deterministic. It invokes no model and opens no network connection, whatever inference is configured.

  Rule: a plan captured on one operator's terminal runs on another's, and terminals differ in size. A rectangle fixed in cells cannot survive that crossing; a role and a name can. The model schema carries which of a region's edges are anchored and which are elastic.

  Rule: terminal size is a property of the run and never of the plan. The caller supplies it, defaulting to the operator's own terminal, and it reaches the PTY and the virtual screen together. A plan that recorded its capture size would invite replay to restore that size, which is the one thing these scenarios exist to prevent.

  Scenario: a plan captured at one terminal width replays at another
    Given a harness plan captured from the fixture terminal program at 80 columns
    When that plan is replayed at 120 columns
    Then the replay passes

  Scenario: a status line stays bound when the terminal widens
    Given a harness plan whose step expects the status bar to contain "READY", captured at 80 columns
    When that plan is replayed at 120 columns
    Then the replay passes

  Scenario: replaying a recorded flow reproduces the interaction
    Given a harness plan driving the fixture terminal program
    When that plan is replayed
    Then the replay passes

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

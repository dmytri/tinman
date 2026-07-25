Feature: replay
  As a test author
  I want a recorded flow replayed exactly
  So that a captured interaction becomes a repeatable test

  Rule: replay is absolutely deterministic. It invokes no model and opens no network connection, whatever inference is configured.

  Rule: a recorded locator binds by role and name, so it survives a terminal that is not the size it was captured at. A region's rectangle records which of its edges are anchored and which are elastic, and the elastic edges move with the viewport while the anchored edges hold their offset. A plan captured on one operator's terminal runs on another's.

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

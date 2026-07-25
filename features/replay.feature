Feature: replay
  As a test author
  I want a recorded flow replayed exactly
  So that a captured interaction becomes a repeatable test

  Rule: replay is absolutely deterministic. It invokes no model and opens no network connection, whatever inference is configured.

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

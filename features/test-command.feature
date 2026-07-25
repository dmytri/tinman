Feature: test command
  As a test author
  I want "tinman test" to run a plan and report the result
  So that a recorded plan runs in continuous integration

  Scenario: a passing plan exits successfully
    Given a harness plan driving the fixture terminal program
    When the operator tests that plan
    Then the command exits with status 0

  Scenario: a failing plan exits with a failure status
    Given a harness plan driving the fixture terminal program whose final step expects the text "Deployed"
    When the operator tests that plan
    Then the command exits with status 1

  Scenario: the failure report names the step and shows the screen
    Given a harness plan driving the fixture terminal program whose final step expects the text "Deployed"
    When the operator tests that plan
    Then the output reports the step expecting "Deployed"
    And the output contains the text "Username"

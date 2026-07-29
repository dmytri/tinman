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

  Rule: the scenarios above cover a plan that ran. A plan that never loads is the more common operator error, and the binary aborts on both of its forms: an unreadable file and a file that does not parse each reach a panic in src/main.rs rather than a reported failure. The operator meets a panic message and a backtrace note, and the process leaves status 101 where every handled failure beside it leaves 1. The panic-free contract at scantlings/panic-free-seams.json states the rule these paths break, and its search paths reach the driver and the assistant rather than the binary every operator runs.

  Scenario: a plan file that is not there is reported rather than fatal
    Given no file named "missing.yaml" exists
    When the operator tests the plan "missing.yaml"
    Then the command exits with status 1
    And the failure names the plan file that was not read
    And the output carries no panic

  Scenario: a plan that does not parse is reported rather than fatal
    Given a plan file that is not valid YAML
    When the operator tests that plan
    Then the command exits with status 1
    And the failure reports the plan did not parse
    And the output carries no panic

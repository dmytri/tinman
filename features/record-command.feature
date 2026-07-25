Feature: record command
  As an operator
  I want "tinman record" to capture a live session to an editable file
  So that I can turn an interaction I performed into a test

  Scenario: record writes an interaction log for the captured program
    When the operator records the command "printf READY" and presses "q"
    Then the written interaction log names the program "printf"
    And the written interaction log lists a key press "q"

  @sandbox
  Scenario: record launches its target inside the sandbox
    Given the operator's environment defines the secret "TINMAN_SECRET" as "hunter2"
    When the operator records the fixture terminal program
    Then the recorded snapshots show the secret value is absent

  Scenario: record refuses to overwrite an existing log
    Given the file "session.yaml" already exists
    When the operator records the command "printf READY" into "session.yaml"
    Then recording fails and reports the file already exists

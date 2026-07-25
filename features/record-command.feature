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

  Rule: a recording that cannot replay itself is a draft, not a recording. Record replays the captured plan deterministically before writing it, so every locator is proved to rebind with no model invocation while the operator is still present to correct it.

  Scenario: a written plan replays the interaction it recorded
    When the operator records the fixture terminal program
    Then replaying the written plan reproduces the recorded interaction

  Scenario: a plan that fails its own replay is not written
    Given a fixture terminal program whose pane titles change between draws
    When the operator records that program
    Then recording fails and reports the plan did not replay
    And no interaction log is written

  Scenario: record writes to a default file when no path is given
    When the operator records the command "printf READY"
    Then the interaction log is written to "tinman.yaml"

  Scenario: record refuses to overwrite an existing log
    Given the file "session.yaml" already exists
    When the operator records the command "printf READY" into "session.yaml"
    Then recording fails and reports the file already exists

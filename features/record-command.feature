Feature: record command
  As an operator
  I want "tinman record" to capture a live session to an editable file
  So that I can turn an interaction I performed into a test

  Rule: record writes one file and that file is a plan. The interaction log is the capture spine's own artifact and `features/interaction-log.feature` owns it; what the operator edits and replays is a plan, as `assets/help/tinman.txt` advertises. The output path is given by the `--output` option and defaults to `tinman.yaml`.

  Scenario: record writes a replayable plan for the captured program
    When the operator records the command "printf READY" and presses "q"
    Then the written plan names the command "printf READY"
    And the written plan records a key press "q"

  @sandbox
  Scenario: record launches its target inside the sandbox
    Given the operator's environment defines the secret "TINMAN_SECRET" as "hunter2"
    When the operator records the fixture terminal program
    Then the recorded snapshots show the secret value is absent

  Rule: a recording that cannot replay itself is a draft, not a recording. The operator is present at capture time and absent at replay time, so a locator proved while they are still here costs a moment, and one that fails after they leave costs a debugging session.

  Scenario: a recorded plan carries an expectation for what the screen showed
    When the operator records the command "printf READY"
    Then the written plan carries an expectation on the text "READY"

  Scenario: a written plan replays the interaction it recorded
    When the operator records the fixture terminal program
    Then replaying the written plan reproduces the recorded interaction

  Scenario: a plan that fails its own replay is not written
    Given a fixture terminal program whose pane titles change between draws
    When the operator records that program
    Then recording fails and reports the plan did not replay
    And no plan is written

  Scenario: record writes to a default file when no path is given
    When the operator records the command "printf READY"
    Then the plan is written to "tinman.yaml"

  Scenario: record writes to the path the output option names
    When the operator records the command "printf READY" with "--output session.yaml"
    Then the plan is written to "session.yaml"

  Scenario: record refuses to overwrite an existing plan
    Given the file "session.yaml" already exists
    When the operator records the command "printf READY" with "--output session.yaml"
    Then recording fails and reports the file already exists

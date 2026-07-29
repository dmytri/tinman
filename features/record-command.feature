Feature: record command
  As an operator
  I want "tinman record" to capture a live session to an editable file
  So that I can turn an interaction I performed into a test

  Rule: record writes one file and that file is a plan. The interaction log is the capture spine's own artifact and `features/interaction-log.feature` owns it; what the operator edits and replays is a plan, as `assets/help/tinman.txt` advertises. The output path is given by the `--output` option and defaults to `tinman.yaml`.

  Rule: a tier tag follows the cost a scenario incurs rather than the subject it asserts on. A scenario about where the plan file lands still launches a program under Bubblewrap to get there, so it carries the sandbox tier's cost whatever it is about. A scenario that fails before any launch, such as the refusal to overwrite, costs nothing and stays in the default tier.

  @sandbox
  Scenario: record writes a replayable plan for the captured program
    When the operator records the command "printf READY" and presses "q"
    Then the written plan names the command "printf READY"
    And the written plan records a key press "q"

  Rule: a recorded snapshot missing the operator's secret says nothing about isolation when the recorded program never tried to print it, and that is how this scenario stayed green while record launched its target with the operator's whole environment inherited. A snapshot missing the secret because nothing ran and one missing it because the sandbox held are the same snapshot, which is what a marker drawn beside the attempt separates.

  @sandbox
  Scenario: record launches its target inside the sandbox
    Given the operator's environment defines the secret "TINMAN_SECRET" as "hunter2"
    When the operator records a command that prints "ran" and the value of "TINMAN_SECRET"
    Then the recorded snapshots show the text "ran"
    And the recorded snapshots show the secret value is absent

  Rule: a recording that cannot replay itself is a draft, not a recording. The operator is present at capture time and absent at replay time, so a locator proved while they are still here costs a moment, and one that fails after they leave costs a debugging session.

  @sandbox
  Scenario: a recorded plan carries an expectation for what the screen showed
    When the operator records the command "printf READY"
    Then the written plan carries an expectation on the text "READY"

  @sandbox
  Scenario: a written plan replays the interaction it recorded
    When the operator records the fixture terminal program
    Then replaying the written plan reproduces the recorded interaction

  @sandbox
  Scenario: a plan that fails its own replay is not written
    Given a fixture terminal program whose pane titles change between draws
    When the operator records that program
    Then recording fails and reports the plan did not replay
    And no plan is written

  @sandbox
  Scenario: record writes to a default file when no path is given
    When the operator records the command "printf READY"
    Then the plan is written to "tinman.yaml"

  @sandbox
  Scenario: record writes to the path the output option names
    When the operator records the command "printf READY" with "--output session.yaml"
    Then the plan is written to "session.yaml"

  Scenario: record refuses to overwrite an existing plan
    Given the file "session.yaml" already exists
    When the operator records the command "printf READY" with "--output session.yaml"
    Then recording fails and reports the file already exists

  Rule: ending a recording sends the end-of-transmission character once every twenty milliseconds until the recorded program exits, and nothing bounds that loop. Every other wait in the recorder carries a ceiling, and the opening screen is taken as it stands once its deadline passes. A program that ignores the end of its input leaves the operator with a command that never returns and a terminal still in raw mode, which is the state the recorder took and only returns after the loop it never leaves.

  Scenario: a recorded program that ignores the end of input ends at its deadline
    Given the operator records a program that never exits when its input ends
    When the operator ends the input
    Then recording fails and reports the session reached its deadline
    And the terminal is out of raw mode

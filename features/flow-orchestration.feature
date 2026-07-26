Feature: flow orchestration
  As a test author
  I want one flow to drive several processes in order
  So that a test can prepare a workspace, drive a terminal program, then check the result

  Scenario: a flow runs its steps in the order written
    Given a flow that runs "printf first > order.txt", then runs "printf second >> order.txt"
    When the flow is executed
    Then the file "order.txt" contains "firstsecond"

  Scenario: a failing run step stops the flow
    Given a flow that runs "false", then runs "printf reached > reached.txt"
    When the flow is executed
    Then execution fails and reports the step that failed
    And the file "reached.txt" does not exist

  @sandbox
  Scenario: a tui step drives its program under the sandbox
    Given a flow whose only step drives the fixture terminal program
    When the flow is executed
    Then the fixture program reports a home directory other than the operator's home

  @sandbox
  Scenario: a run step executes its command under the sandbox
    Given a flow whose only step runs "printf %s $HOME"
    When the flow is executed
    Then the step reports a home directory other than the operator's home

  Scenario: a run step sees the workspace the previous step wrote
    Given a flow that runs "printf hello > shared.txt", then runs "cat shared.txt"
    When the flow is executed
    Then the second step's output is "hello"

  Rule: a run step and a tui step observe their process differently because a pseudo-terminal merges the two output streams into one and cannot separate them again. A run step reads pipes, so a command-line program's streams and exit status stay distinguishable. A tui step reads a pseudo-terminal, which is what a full-screen program needs to draw at all. The same program often wants both: a coding agent answering one prompt is a command-line program, and the same binary run interactively is a full-screen one.

  Scenario: a run step keeps standard error apart from standard output
    Given a flow that runs "printf out; printf err >&2"
    When the flow is executed
    Then the step's standard output is "out"
    And the step's standard error is "err"

  Scenario: a run step reports the status its command exited with
    Given a flow whose only step runs "exit 3" and expects the status 3
    When the flow is executed
    Then the flow passes

  Scenario: an unexpected exit status fails the flow
    Given a flow whose only step runs "exit 3" and expects the status 0
    When the flow is executed
    Then execution fails and reports the status 3

  Scenario: a run step feeds its command the input the plan carries
    Given a flow whose only step runs "cat" with the input "hello"
    When the flow is executed
    Then the step's standard output is "hello"

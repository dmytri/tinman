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

  Scenario: a run step sees the workspace the previous step wrote
    Given a flow that runs "printf hello > shared.txt", then runs "cat shared.txt"
    When the flow is executed
    Then the second step's output is "hello"

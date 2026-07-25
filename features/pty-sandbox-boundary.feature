Feature: pty sandbox boundary
  As a Tinman maintainer
  I want the PTY runner to stay free of backend knowledge
  So that new sandbox backends never change the PTY architecture

  @contract
  Scenario: the PTY runner discharges the sandbox-boundary contract
    Given the PTY runner source
    When the verifier checks the PTY sandbox boundary
    Then no counterexample is found

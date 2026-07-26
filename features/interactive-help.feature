Feature: interactive help
  As an operator
  I want to ask Tinman for the command I need
  So that I can act without reading the whole manual

  Rule: the assistant has a deliberately narrow scope. It answers questions about Tinman and proposes Tinman commands. Every action it proposes runs through Tinman's normal command parser, so it can never become a general-purpose shell.

  Scenario: the assistant is appended beneath the conventional help
    Given inference is available
    When the operator runs "tinman --help" in an interactive terminal
    Then the help output ends with the body of the asset at "assets/help/assistant-prompt.txt"

  Scenario: a question typed at the prompt is answered
    Given inference is available
    And the assistant answers "Replay runs a recorded plan with no model."
    And the operator runs "tinman --help" in an interactive terminal
    When the operator types "what does replay do" at the assistant prompt
    Then the output displays the answer "Replay runs a recorded plan with no model."

  Scenario: the assistant session ends when the operator ends the input
    Given inference is available
    And the operator runs "tinman --help" in an interactive terminal
    When the operator ends the input
    Then the command exits with status 0

  Scenario: a proposed command is displayed before it runs
    Given the assistant infers the command "tinman record opencode"
    When the operator asks "record the opencode agent"
    Then the assistant displays the proposed command "tinman record opencode"
    And the proposed command has not run

  Scenario: a declined proposal does not run
    Given the assistant has proposed the command "tinman record opencode"
    When the operator declines the proposal
    Then the proposed command has not run

  Scenario: a confirmed proposal runs through the command parser
    Given the assistant has proposed the command "tinman record opencode"
    When the operator confirms the proposal
    Then the command parser receives the arguments "record" and "opencode"

  Scenario: a proposal outside Tinman's command set is refused
    Given the assistant infers the command "rm -rf /"
    When the operator asks "delete everything"
    Then the assistant refuses the proposal
    And no command is offered to the operator

  Scenario: a proposal naming an unknown Tinman subcommand is refused
    Given the assistant infers the command "tinman teleport opencode"
    When the operator asks "teleport the opencode agent"
    Then the assistant refuses the proposal

  Scenario: a question is answered without proposing a command
    Given the assistant answers "Replay runs a recorded plan with no model."
    When the operator asks "what does replay do"
    Then the assistant displays the answer "Replay runs a recorded plan with no model."
    And no command is offered to the operator

  @contract
  Scenario: the assistant discharges the command-parser boundary contract
    Given the interactive assistant source
    When the verifier checks the assistant command boundary
    Then no counterexample is found

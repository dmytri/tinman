Feature: command dispatch
  As an operator
  I want every command Tinman advertises to do what it says
  So that a command line I read in the help text is one I can actually run

  Rule: the parser and the dispatch are two lists that must agree, and nothing but the compiler can be trusted to keep them agreeing. A command declared in the parser appears in the help text, is accepted by the command parser, and is proposed by the interactive assistant, all before any implementation exists. The set itself is named once, in `features/command-surface.feature`, because a command set stated in two features is two contracts that drift apart, and this one did: the exact set here still read five commands on the voyage that took the parser to seven.

  @contract
  Scenario: the command dispatch discharges its completeness contract
    Given the command dispatch source
    When the verifier checks the command dispatch completeness
    Then no counterexample is found

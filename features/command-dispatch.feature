Feature: command dispatch
  As an operator
  I want every command Tinman advertises to do what it says
  So that a command line I read in the help text is one I can actually run

  Rule: the parser and the dispatch are two lists that must agree, and nothing but the compiler can be trusted to keep them agreeing. A command declared in the parser appears in the help text, is accepted by the command parser, and is proposed by the interactive assistant, all before any implementation exists. The named set below is the floor: a command added to the parser without being named here reddens rather than shipping.

  Scenario: the parser accepts exactly the implemented commands
    Given the commands the parser accepts
    When the accepted command set is read
    Then it is exactly "record", "test", "inspect", "driver" and "help"

  @contract
  Scenario: the command dispatch discharges its completeness contract
    Given the command dispatch source
    When the verifier checks the command dispatch completeness
    Then no counterexample is found

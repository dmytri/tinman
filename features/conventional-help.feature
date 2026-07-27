Feature: conventional help
  As an operator
  I want "tinman --help" to describe the whole command line
  So that I can use Tinman without any inference

  Rule: the help text is a human-owned asset inlined at build time. It carries one "{{tagline}}" placeholder line, which inference fills and which is removed when nothing fills it. Conventional help is authoritative and stays fully usable with no model, no credential and no network.

  Scenario: help renders the bundled help text
    Given inference is available
    When the operator runs "tinman --help" with stdout redirected to a file
    Then the help output is the asset at "assets/help/tinman.txt" with the tagline line removed

  Rule: the help subcommand and the help flag are one behaviour reached two ways, so both render the bundled asset. A parser that generates its own help for the subcommand renders the doc comments attached to the command types instead, and those doc comments are where the trace annotations live, so an operator asking for help is shown the project's own planks.

  Scenario: the help subcommand renders the bundled help text
    Given inference is available
    When the operator runs "tinman help" with stdout redirected to a file
    Then the help output is the asset at "assets/help/tinman.txt" with the tagline line removed

  Rule: the commands are looked for in the help text's Commands block rather than anywhere in the file. A command name is an ordinary English word that the surrounding prose uses freely, so a whole-file search reports a command as documented when only the prose mentions it. That is not hypothetical: while the parser still accepted "replay" and the Commands block had already dropped it, a whole-file search passed on the closing sentence about replay time being deterministic.

  Scenario: every command the parser accepts is listed in the help text's Commands block
    Given the commands the parser accepts
    When each is looked for in the Commands block of the asset at "assets/help/tinman.txt"
    Then every accepted command is listed in the Commands block

  Rule: the check above reads one direction only, and the other direction is where `replay` did its damage. A command listed in the block and missing from the parser is a command the help text tells an operator to run and Tinman refuses, which is the same broken promise as a command the parser accepts and the help omits. One direction was checked for a year while the other was the one that shipped.

  Scenario: every command the help text's Commands block lists is accepted by the parser
    Given the commands listed in the Commands block of the asset at "assets/help/tinman.txt"
    When each is passed to the command parser
    Then the parser accepts every listed command
    And the commands read are not empty

  Scenario: every option the help text advertises is accepted by the parser
    Given the options the asset at "assets/help/tinman.txt" advertises
    When each is passed to the command parser
    Then the parser accepts every advertised option

  Rule: an example is the part of a help text a reader actually types, so it is the part that costs them most when it has drifted. The Commands block is checked in both directions and an example line escapes both, because it carries flags, arguments and a subcommand together in the one form the parser will really be handed. The tldr-pages style guide is the source for the shape: simplest invocation first, complexity introduced gradually, and around five of them.

  Scenario: every example the help text carries is accepted by the parser
    Given the Tinman command lines in the Examples block of the asset at "assets/help/tinman.txt"
    When each is passed to the command parser
    Then the parser accepts every command line
    And the command lines read are not empty

  Scenario: the help text carries exactly one tagline placeholder
    Given the asset at "assets/help/tinman.txt"
    When its tagline placeholders are counted
    Then the count is 1

  Scenario: help exits successfully
    When the operator executes "tinman --help"
    Then the command exits with status 0

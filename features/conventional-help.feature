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

  Rule: the tagline is the one line of the help a model writes, and it is the only line worth waiting on. Rendering the rest first and letting the tagline arrive into place keeps the help instant whatever the provider is doing, and makes the generated line legible as generated. The failure this shape must not have is a spinner that outlives its answer: an animation with no end state is worse than a missing line, because a missing line is honest and a spinner promises something is still coming. The tagline ceiling is that end state, and the scenario below is what proves it exists.

  Rule: motion is a presentation channel like colour, so it carries no meaning of its own here and nothing is lost by never seeing it. A terminal gets the animation, a redirected stream gets none, and a provider that never answers gets a settled line rather than a spinner. That keeps the help readable by a screen reader, a pipe and an operator on a slow link alike.

  Scenario: the help is on the screen before the tagline arrives
    Given inference is available
    When the operator runs "tinman --help" in an interactive terminal
    Then the Commands block is drawn before the tagline is drawn

  Scenario: the tagline arrives into place when inference answers
    Given inference is available
    When the operator runs "tinman --help" in an interactive terminal
    Then the tagline line carries the generated expansion

  Scenario: a tagline that never arrives settles rather than spinning
    Given the inference provider does not answer
    When the operator runs "tinman --help" in an interactive terminal
    Then the tagline line settles inside the tagline ceiling
    And no spinner frame is on the screen

  Scenario: redirected help waits for no tagline
    Given inference is available
    When the operator runs "tinman --help" with stdout redirected to a file
    Then the Commands block is in the output
    And the output carries no spinner frame

  Rule: the examples are authored in the tldr-pages placeholder syntax and expanded before an operator sees them, so one source serves two audiences. Unexpanded, the block is the body of a tldr page and stays in step with the parser instead of drifting into a separately written document. Expanded, it is a command a reader can paste. The asset already worked this way for "{{tagline}}", so this is that mechanism reaching the examples rather than a second placeholder vocabulary. Expansion is what keeps Tinman's own documentation honest by its own standard: an unexpanded placeholder is a template, and inspecting a program whose help carries templates reports them untestable as written, which is this project's signature fault aimed at its own flagship example.

  Scenario: the rendered help carries no unexpanded placeholder
    When the operator runs "tinman --help" in an interactive terminal
    Then the Examples block is on the screen
    And no placeholder delimiter is on the screen

  Scenario: every example placeholder has a value to expand to
    Given the placeholders in the Examples block of the asset at "assets/help/tinman.txt"
    When each is looked up among the example values
    Then every one has a value
    And the placeholders read are not empty

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

Feature: conventional help
  As an operator
  I want "tinman --help" to describe the whole command line
  So that I can use Tinman without any inference

  Rule: the help text is a human-owned asset inlined at build time. It carries one "{{tagline}}" placeholder line, which inference fills and which is removed when nothing fills it. Conventional help is authoritative and stays fully usable with no model, no credential and no network.

  Scenario: help renders the bundled help text
    Given inference is available
    When the operator runs "tinman --help" with stdout redirected to a file
    Then the help output is the asset at "assets/help/tinman.txt" with the tagline line removed

  Scenario: every command the parser accepts appears in the help text
    Given the commands the parser accepts
    When each is looked for in the asset at "assets/help/tinman.txt"
    Then every accepted command appears in the help text

  Scenario: the help text carries exactly one tagline placeholder
    Given the asset at "assets/help/tinman.txt"
    When its tagline placeholders are counted
    Then the count is 1

  Scenario: help exits successfully
    When the operator executes "tinman --help"
    Then the command exits with status 0

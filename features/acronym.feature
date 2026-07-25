Feature: acronym expansion
  As an operator
  I want a fresh TINMAN expansion in the help output
  So that the tagline line carries whatever the model made of the skill

  Rule: the expansion is cosmetic and is never canonical; different invocations may produce different expansions. The generator is given the bundled skill's name and description and nothing else, and whatever it returns fills the tagline. Tinman constrains neither the wording nor the shape of the expansion.

  Scenario: the generated expansion fills the tagline line
    Given a provider that returns "Terminal Inference Navigating Model Agent Networks"
    When the operator runs "tinman --help" in an interactive terminal
    Then the tagline line is "Terminal Inference Navigating Model Agent Networks"

  Scenario: an empty generation falls back to the unavailable notice
    Given a provider that returns an empty response
    When the operator runs "tinman --help" in an interactive terminal
    Then the tagline line is the body of the asset at "assets/help/inference-unavailable.txt"

Feature: acronym expansion
  As an operator
  I want the TINMAN expansion validated before it is displayed
  So that a malformed expansion never reaches the help output

  Rule: the expansion carries six acronym words whose initials spell TINMAN. Lowercase connective words such as "for" and "of" may sit between them and take no initial. The expansion is cosmetic and is never canonical; different invocations may produce different expansions.

  Scenario: a well-formed expansion is accepted
    Given the expansion "Terminal Inference for Navigating Model Agent Networks"
    When the expansion is validated
    Then the expansion is accepted

  Scenario: an expansion with no connective words is accepted
    Given the expansion "Terminal Interfaces Navigated Mechanically Against Noise"
    When the expansion is validated
    Then the expansion is accepted

  Scenario: an expansion whose initials do not spell TINMAN is rejected
    Given the expansion "Terminal Inference for Running Model Agent Networks"
    When the expansion is validated
    Then the expansion is rejected

  Scenario: an expansion with too few acronym words is rejected
    Given the expansion "Terminal Inference Navigating Model Agents"
    When the expansion is validated
    Then the expansion is rejected

  Scenario: an expansion carrying punctuation is rejected
    Given the expansion "Terminal, Inference, Navigating Model Agent Networks"
    When the expansion is validated
    Then the expansion is rejected

  Scenario: an expansion spanning two lines is rejected
    Given the expansion "Terminal Inference Navigating" followed by a newline and "Model Agent Networks"
    When the expansion is validated
    Then the expansion is rejected

  Scenario: generation retries once after an invalid expansion
    Given a provider that returns "not a valid expansion" then "Terminal Inference Navigating Model Agent Networks"
    When an acronym expansion is generated
    Then the generated expansion is "Terminal Inference Navigating Model Agent Networks"

  Scenario: two invalid expansions yield no acronym
    Given a provider that returns "not a valid expansion" twice
    When an acronym expansion is generated
    Then no expansion is produced

  Scenario: a valid expansion fills the tagline line
    Given a provider that returns "Terminal Inference Navigating Model Agent Networks"
    When the operator runs "tinman --help" in an interactive terminal
    Then the tagline line is "Terminal Inference Navigating Model Agent Networks"

  Scenario: a failed generation falls back to the unavailable notice
    Given a provider that returns "not a valid expansion" twice
    When the operator runs "tinman --help" in an interactive terminal
    Then the tagline line is the body of the asset at "assets/help/inference-unavailable.txt"

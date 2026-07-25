Feature: plan shorthand
  As a test author
  I want a trivial test to stay trivial and a detailed test to stay expressible
  So that the plan format grows with what the test needs

  Rule: every shorthand normalizes into the one canonical plan. A shorthand removes typing; it never adds capability, and it never weakens a default.

  Scenario: a bare command with steps is a single-process flow
    Given the harness plan:
      """
      tui: printf READY
      steps:
        - expect: READY
      """
    When the plan is parsed
    Then the plan's flow holds 1 step
    And the flow's first step drives the command "printf READY"

  Scenario: a scalar expectation normalizes to a text expectation
    Given a harness plan whose only step is "expect: Saved"
    When the plan is parsed
    Then the step expects the text "Saved"

  Scenario: a scalar activation normalizes to a name locator
    Given a harness plan whose only step is "activate: Settings"
    When the plan is parsed
    Then the step's locator name is "Settings"
    And the step's locator names no role

  Scenario: a plan with no sandbox section is sandboxed by default
    Given the harness plan:
      """
      tui: printf READY
      steps:
        - expect: READY
      """
    When the plan is parsed
    Then the parsed sandbox specification denies network access
    And the parsed sandbox specification provisions an empty home

  Scenario: the shorthand form and the full form parse to the same plan
    Given the harness plan at "assets/examples/settings-flow.yaml"
    And the harness plan at "assets/examples/settings-flow-shorthand.yaml"
    When both plans are parsed
    Then the two parsed plans are identical

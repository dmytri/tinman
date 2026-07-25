Feature: harness plan
  As a test author
  I want a recorded flow stored as constrained YAML
  So that I can read and edit a captured test by hand

  Rule: the YAML plan is the canonical representation of a recorded flow. Tinman offers no programming-language test DSL; a test written in another language drives Tinman through the driver protocol instead.

  @contract
  Scenario: the example flow conforms to the harness schema
    Given the harness plan at "assets/examples/settings-flow.yaml"
    When the plan is parsed
    Then it conforms to the "harness-plan" schema in "scantlings/harness-plan.schema.json"

  Scenario: a plan step naming an unknown keyword is rejected
    Given a harness plan whose first step uses the keyword "teleport"
    When the plan is parsed
    Then parsing fails and reports the unknown step keyword "teleport"

  Scenario: a plan with no flow is rejected
    Given a harness plan that defines no flow
    When the plan is parsed
    Then parsing fails and reports a missing flow

  Scenario: the plan's sandbox section is backend-neutral
    Given the harness plan at "assets/examples/settings-flow.yaml"
    When the plan is parsed
    Then the parsed sandbox specification names no Bubblewrap flag

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

  Rule: the scenario above reaches the step keyword and stops there. A field inside a step was accepted and dropped, so a step carrying a field the model does not define parsed, ran, asserted nothing and exited zero. Tinman exists so a claim about a program is checked rather than believed, and an assertion that disappears while the run reports success is that promise inverted, landing on the operator's own suite rather than on this one. The contract below bounds the modules that parse operator-authored YAML, because the step or locator a later feature adds is where the next silent drop would hide. Three of this project's own sandbox fixtures were staging a removed field the same way, read as passing, and were found the day the strict parser landed.

  Scenario: a plan step naming an unknown field is rejected
    Given a harness plan whose run step carries the field "staus" where "status" was meant
    When the plan is parsed
    Then parsing fails and reports the unknown field "staus"

  @contract
  Scenario: every type parsing an operator's plan refuses an unknown field
    Given the implementation sources
    When the verifier checks the plan deserialization strictness contract
    Then no counterexample is found

  Scenario: a plan with no flow is rejected
    Given a harness plan that defines no flow
    When the plan is parsed
    Then parsing fails and reports a missing flow

  Scenario: the plan's sandbox section is backend-neutral
    Given the harness plan at "assets/examples/settings-flow.yaml"
    When the plan is parsed
    Then the parsed sandbox specification names no Bubblewrap flag

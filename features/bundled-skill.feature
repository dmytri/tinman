Feature: bundled skill
  As an operator
  I want Tinman to ship exactly one skills.sh-compatible skill
  So that coding agents and Tinman itself read the same documentation

  Rule: the bundled skill is the single source of truth for what Tinman is and how it is used. It has three consumers: an external coding agent reads the whole skill, "tinman --help" loads the whole skill to answer questions, and the acronym generator reads its name and description.

  @contract
  Scenario: the bundled skill conforms to the skill schema
    Given the bundled skill at "assets/skill/SKILL.md"
    When the skill front matter is parsed
    Then it conforms to the "skill" schema in "scantlings/skill.schema.json"

  Scenario: Tinman loads the skill it ships
    Given the bundled skill at "assets/skill/SKILL.md"
    When Tinman loads its bundled skill
    Then the loaded skill body is identical to the file's body

  Scenario: the acronym context instructs the model before naming the skill
    Given the bundled skill at "assets/skill/SKILL.md"
    When the acronym context is built
    Then the context begins with the body of the asset at "assets/help/acronym-prompt.txt"
    And the context carries the skill's "name" and "description" fields

  Scenario: the assistant answers from the whole bundled skill
    Given the bundled skill at "assets/skill/SKILL.md"
    When the assistant context is built
    Then the context contains the skill body

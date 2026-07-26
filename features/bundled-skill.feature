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

  Rule: the bundled skill is the whole context the assistant answers from, so a command line it names is a command an operator will be told to run. The skill and the parser are the same two-list problem the help text has, one list further from the code, and the skill ships to coding agents rather than to a reader who can check. A command struck from the parser stayed in this asset through the voyage that struck it, and nothing reddened.

  Scenario: every command the bundled skill names is a command the parser accepts
    Given the command lines in the asset at "assets/skill/SKILL.md"
    When each named command is passed to the command parser
    Then the parser accepts every command the skill names

  Scenario: Tinman loads the skill it ships
    Given the bundled skill at "assets/skill/SKILL.md"
    When Tinman loads its bundled skill
    Then the loaded skill body is identical to the file's body

  Scenario: the acronym context instructs the model before naming the skill
    Given the bundled skill at "assets/skill/SKILL.md"
    When the acronym context is built
    Then the context begins with the body of the asset at "assets/help/acronym-prompt.txt"

  Rule: the skill tells the model what Tinman is; it does not tell it how to answer. An operator at a terminal wants one line and a command to run, and a model given only reference material answers at the length of the reference material. How to answer is therefore its own asset, carried ahead of the skill in the same way the acronym context carries its own instruction.

  Scenario: the assistant context instructs the model how to answer before naming the skill
    Given the bundled skill at "assets/skill/SKILL.md"
    When the assistant context is built
    Then the context begins with the body of the asset at "assets/help/assistant-instruction.txt"
    And the context carries the skill's "name" and "description" fields

  Scenario: the assistant answers from the whole bundled skill
    Given the bundled skill at "assets/skill/SKILL.md"
    When the assistant context is built
    Then the context contains the skill body

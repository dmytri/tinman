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

  Rule: the skill tells the model what Tinman is; it does not tell it how to answer. An operator at a terminal wants one line and a command to run, and a model given only reference material answers at the length of the reference material. How to answer is therefore its own asset, carried ahead of the skill in the same way the acronym context carries its own instruction.

  Scenario: the assistant context instructs the model how to answer before naming the skill
    Given the bundled skill at "assets/skill/SKILL.md"
    When the assistant context is built
    Then the context begins with the body of the asset at "assets/help/assistant-instruction.txt"

  Scenario: the assistant answers from the whole bundled skill
    Given the bundled skill at "assets/skill/SKILL.md"
    When the assistant context is built
    Then the context contains the skill body

  Rule: a scantling is the exact shape a plan, a protocol message or a model document must take, which is the thing an operator gets wrong. It also cannot drift from the product, because a @contract scenario pins each one, where the skill can drift and has. The specs are the other candidate: implementer-facing, half of them attesting the project's own method rather than the product, and four times the size of the scantlings on a path that already waits tens of seconds a turn.

  Scenario: the assistant context carries every user-facing scantling
    Given the bundled skill at "assets/skill/SKILL.md"
    When the assistant context is built
    Then the context carries the body of each of these scantlings
      | scantling                              |
      | scantlings/harness-plan.schema.json    |
      | scantlings/sandbox-spec.schema.json    |
      | scantlings/tom.schema.json             |
      | scantlings/driver-protocol.schema.json |
      | scantlings/interaction-log.schema.json |
      | scantlings/skill.schema.json           |

  Scenario: the assistant context carries no specification
    Given the scenario names in the specs
    When the assistant context is built
    Then the context carries no scenario name from the specs
    And the scenario names read are not empty

  Rule: the skill teaches by example, and it is the one document an external coding agent reads to learn Tinman. An example that no longer holds is worse than no example, because the agent writes what it was shown and the failure surfaces in their project rather than in ours. This is not hypothetical: the skill named nine roles the model has never produced and a wire shape the driver has never spoken, and both survived every green run this suite has ever had. Checking the examples against the scantlings they claim to illustrate is the same join the parser check makes for command lines, over the other two things the skill teaches.

  Scenario: every role the bundled skill names is a role the model produces
    Given the roles the asset at "assets/skill/SKILL.md" names
    When each is matched against the roles the "tom" schema declares
    Then every role the skill names is a role the model produces
    And the roles read are not empty

  Scenario: every driver message in the bundled skill conforms to the protocol
    Given the JSON messages in the fenced blocks of the asset at "assets/skill/SKILL.md"
    When each is checked against the "driver-protocol" schema in "scantlings/driver-protocol.schema.json"
    Then every message conforms
    And the messages read are not empty

  Scenario: every plan in the bundled skill conforms to the plan schema
    Given the YAML plans in the fenced blocks of the asset at "assets/skill/SKILL.md"
    When each is checked against the "harness-plan" schema in "scantlings/harness-plan.schema.json"
    Then every plan conforms
    And the plans read are not empty

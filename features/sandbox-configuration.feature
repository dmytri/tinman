Feature: sandbox configuration
  As a security-conscious test author
  I want the plan's sandbox section to control what the target can reach
  So that a test grants exactly what its target needs and nothing else

  Rule: Tinman runs coding agents, which execute arbitrary commands. Isolation is granted explicitly and never inherited.

  Scenario: a mount is read-only unless the plan says otherwise
    Given a plan sandbox section mounting "./fixtures/project" at "/workspace" with no mode
    When the sandbox specification is parsed
    Then the mount's mode is "readonly"

  @sandbox
  Scenario: a copy mount gives the target a writable copy
    Given a plan sandbox section mounting "./fixtures/project" at "/workspace" with mode "copy"
    And the fixture directory "./fixtures/project" contains the file "README"
    When the fixture terminal program writes "changed" into "/workspace/README"
    Then the file "./fixtures/project/README" is unchanged

  @sandbox
  Scenario: a secret is injected only when the plan names it
    Given the operator's environment defines "OPENAI_API_KEY" as "sk-operator"
    And a plan sandbox section that names no environment variables
    When a process is prepared and launched
    Then the launched process reports "OPENAI_API_KEY" is unset

  @sandbox
  Scenario: a named secret reaches the sandboxed process
    Given the operator's environment defines "OPENAI_API_KEY" as "sk-operator"
    And a plan sandbox section that injects "OPENAI_API_KEY" from the host
    When a process is prepared and launched
    Then the launched process reports "OPENAI_API_KEY" is "sk-operator"

  @sandbox
  Scenario: the sandbox PATH holds only the entries the plan lists
    Given a plan sandbox section whose path lists "/usr/bin"
    When a process is prepared and launched
    Then the launched process reports its PATH is "/usr/bin"

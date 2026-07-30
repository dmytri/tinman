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

  Rule: the seam granting a host variable read the process environment table directly, so the only way to stage a grant was to mutate that table. One table serves the whole process and the default tier runs sixty-four workers, so staging a grant changed what every concurrent scenario read, at whatever moment the scheduler interleaved them. That is the failure that reruns clean and teaches a team to rerun. The seam takes the environment it is handed, and the binary passes the process table in at its own edge, where nothing runs beside it.

  Scenario: a host grant reads the environment it is handed
    Given a plan sandbox section that injects "OPENAI_API_KEY" from the host
    And the backend is handed an environment defining "OPENAI_API_KEY" as "sk-handed"
    When the Bubblewrap backend prepares the process
    Then the prepared process carries "OPENAI_API_KEY" as "sk-handed"

  @sandbox
  Scenario: the sandbox PATH holds only the entries the plan lists
    Given a plan sandbox section whose path lists "/usr/bin"
    When a process is prepared and launched
    Then the launched process reports its PATH is "/usr/bin"

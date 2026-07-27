Feature: sandboxed launch
  As a security-conscious operator
  I want a launched program isolated from my home, secrets and network
  So that running an untrusted coding agent is safe by default

  @sandbox
  Scenario: a sandboxed program cannot see the operator's home or secrets
    Given the operator's environment defines the secret "TINMAN_SECRET" as "hunter2"
    And a Bubblewrap-prepared process that prints its home directory and the value of "TINMAN_SECRET"
    When the process is captured through a PTY
    Then the virtual screen shows the secret value is absent
    And the virtual screen shows a home directory other than the operator's home

  @sandbox
  Scenario: a sandboxed program has no network access
    Given a Bubblewrap-prepared process that probes for a network route
    When the process is captured through a PTY
    Then the virtual screen shows the network probe found no route

  Rule: the two scenarios above prepare their own process through the backend, so they reach the backend's treatment of what it prepares and never the commands an operator types. That gap shipped: record and inspect each built a prepared process directly and launched the operator's shell with their real home, environment, path and network, while these scenarios stayed green and the project's own onboarding document described record as sandboxing its target.

  Rule: a scenario asserting that a launch went through the Bubblewrap backend reaches the plumbing being called, and reaches it just as well when the sandbox lets everything through. What separates a contained program from an uncontained one is a file the target was told to write and could not, which is why a sentinel path appears below. An absent file also describes a target that never ran, which is why some output the target drew appears beside it.

  @sandbox
  Scenario: inspect cannot write outside the sandbox
    Given a sentinel path outside the sandbox where no file exists
    When the operator inspects a command that writes to the sentinel path and prints "ran"
    Then the inspect output lists a region named "ran"
    And no file exists at the sentinel path

  @sandbox
  Scenario: record cannot write outside the sandbox
    Given a sentinel path outside the sandbox where no file exists
    When the operator records a command that writes to the sentinel path and prints "ran"
    Then the written plan carries an expectation on the text "ran"
    And no file exists at the sentinel path

  Rule: a prepared process is the only input the PTY runner launches, so whoever constructs one settles what isolation the launched program gets. A per-module reference contract reaches the modules someone remembered to write one for, and the modules that bypassed the backend were exactly the ones nobody had. A bound over the whole implementation tree is what leaves a command added later covered by a contract nobody edited.

  @contract
  Scenario: only the sandbox backend constructs a prepared process
    Given the implementation sources
    When the verifier checks the prepared-process construction boundary
    Then no counterexample is found

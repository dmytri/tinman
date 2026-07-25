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

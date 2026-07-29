Feature: prepared process
  As a Tinman maintainer
  I want a prepared process to hold everything the PTY runner needs
  So that the PTY runner launches without knowing any backend

  @contract
  Scenario: a prepared process conforms to its schema
    Given a command specification for "printf hello"
    When the Bubblewrap backend prepares the process
    Then the prepared process conforms to the "prepared-process" schema in "scantlings/prepared-process.schema.json"

  Rule: the schema requires env, cwd and cleanup, and the one construction site fills all three with empty values, so the conformance scenario above passes on a shape whose three remaining fields are inert. An empty array conforms exactly as a populated one does. The scantling at scantlings/pty-sandbox-boundary.json already names this failure in its own rationale, that a field the prepared process declares and the runner never reads is the same divergence as a field it does not declare at all, and it guards prepared.cwd alone. The scenarios below put the other two fields under the same test.

  Scenario: the prepared process reports the environment the sandbox grants
    Given a sandbox specification granting "TINMAN_SECRET" from the host
    And the operator's environment defines the secret "TINMAN_SECRET" as "hunter2"
    When the Bubblewrap backend prepares the process
    Then the prepared process names "TINMAN_SECRET" among its environment pairs
    And the launched program reads "hunter2" from that variable

  Scenario: the prepared process names the staging directory a copy mount created
    Given a sandbox specification mounting the fixture tree in "copy" mode
    When the Bubblewrap backend prepares the process
    Then the prepared process names that staging directory among the resources to reclaim
    And that directory no longer exists once the process has ended

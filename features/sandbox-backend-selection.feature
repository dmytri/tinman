Feature: sandbox backend selection
  As a Tinman maintainer
  I want backend selection to be explicit and safe
  So that a coding agent never runs unsandboxed by accident

  Rule: sandboxed execution is the default and unsandboxed execution is opt-in

  Scenario: auto selects Bubblewrap on Linux
    Given the requested backend is "auto"
    When the backend is resolved on Linux
    Then the resolved backend is "bubblewrap"

  Scenario: the macOS backend is not yet implemented
    Given the requested backend is "mac"
    When the backend is resolved
    Then resolution fails with an unsupported-backend error

  Scenario: the unsafe local backend requires an explicit opt-in
    Given the requested backend is "none"
    When the backend is resolved without the unsafe option
    Then resolution fails and reports that the unsafe option is required

  Scenario: an unavailable Bubblewrap is a hard failure
    Given the requested backend is "bubblewrap"
    And the Bubblewrap executable is absent
    When a process is prepared and launched
    Then launching fails and reports Bubblewrap is unavailable
    And no unsandboxed process is started

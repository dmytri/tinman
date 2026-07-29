Feature: sandbox backend selection
  As a Tinman maintainer
  I want Tinman to choose the sandbox backend for the platform it runs on
  So that no program is ever launched outside a sandbox

  Rule: the backend is Tinman's choice and never the plan's. A plan author says what the program under test needs, not what isolates it, so the sandbox specification carries no backend field and a plan naming one is refused by the schema rather than ignored. What the abstraction earns is a place for a second backend to land: macOS needs one of its own, and resolution is the seam that keeps adding it from being four edits at four launch sites.

  Rule: there is no unsandboxed route. Construction of the one type the PTY runner launches is already bounded to the Bubblewrap backend over the whole implementation tree, so the property holds structurally rather than by an option nobody may set. The enum carried a "none" value and the resolver an unsafe parameter, and neither had a caller; an escape hatch a published schema advertises is a promise to the operator whether or not the code behind it exists.

  Scenario: the backend resolved for Linux is Bubblewrap
    Given the running platform is Linux
    When the backend is resolved for that platform
    Then the resolved backend is "bubblewrap"

  Scenario: the backend resolved for macOS reports the platform is not yet served
    Given the running platform is macOS
    When the backend is resolved for that platform
    Then resolution fails with an unsupported-backend error
    And the failure names the platform it could not serve

  Scenario: an unavailable Bubblewrap is a hard failure
    Given the requested backend is "bubblewrap"
    And the Bubblewrap executable is absent
    When a process is prepared and launched
    Then launching fails and reports Bubblewrap is unavailable
    And no unsandboxed process is started

  @contract
  Scenario: every launch path reaches its backend through resolution
    Given the implementation sources
    When the verifier checks the backend construction boundary
    Then no counterexample is found

  Rule: the condemnation this replaces could not route. Removing the unsandboxed variant, its resolution error and the unsafe parameter needs an edit to verification support, which is not a harbour write scope, so the removal work order sat in a tag that blocks the voyage and no role could discharge it. A scenario is the form that routes: it names the property the operator ruled, it reddens while the code contradicts it, and it goes green when the code is gone. An absence a tag asserts is an absence nobody runs.

  Scenario: resolution offers no unsandboxed outcome
    Given the backend resolution seam
    When every outcome resolution can return is enumerated
    Then every outcome names a sandbox backend

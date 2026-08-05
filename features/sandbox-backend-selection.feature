Feature: sandbox backend selection
  As a Tinman maintainer
  I want Tinman to choose the sandbox backend for the platform it runs on
  So that no program is ever launched outside a sandbox

  Rule: a plan author says what the program under test needs, not what isolates it. The sandbox specification carried a backend field naming four values while no launch path read any of them, so a plan could name one and get Bubblewrap regardless. What the abstraction earns once that field is gone is a place for a second backend to land: macOS needs one of its own, and resolution is the seam that keeps adding it from being four edits at four launch sites.

  Rule: construction of the one type the PTY runner launches is bounded to the Bubblewrap backend over the whole implementation tree, which is where the absence of an unsandboxed route already rested. The enum carried a "none" value and the resolver an unsafe parameter, and neither had a caller; an escape hatch a published schema advertises is a promise to the operator whether or not code stands behind it. The scenario below is what makes that absence checkable rather than incidental.

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

  Rule: the checker matched the type as the contract spells it, so a construction written with its full path slipped past while this scenario stayed green. A plant found it and no count could: the backend module supplies a legitimate construction, so a not-empty floor reads satisfied whatever any other module does. The floor that catches it names the spelling rather than the tally, which is the shape a floor has to take whenever one permitted use keeps the count honest on its own.

  @contract
  Scenario: every launch path reaches its backend through resolution
    Given the implementation sources
    When the verifier checks the backend construction boundary
    Then no counterexample is found
    And a construction written with a qualified path is counted the same as a bare one

  Rule: the condemnation this replaces could not route. Removing the unsandboxed variant, its resolution error and the unsafe parameter needs an edit to verification support, which is not a harbour write scope, so the removal work order sat in a tag that blocks the voyage and no role could discharge it. A scenario is the form that routes: it names the property the operator ruled, it reddens while the code contradicts it, and it goes green when the code is gone. An absence a tag asserts is an absence nobody runs.

  Scenario: resolution offers no unsandboxed outcome
    Given the backend resolution seam
    When every outcome resolution can return is enumerated
    Then every outcome names a sandbox backend

Feature: methodology conformance
  As a Tinman maintainer
  I want methodology breaches to surface as failing verification
  So that a violation is discovered by a run rather than by eye

  Rule: these scenarios attest the project's own method, not the product. They leave with the method they guard. The rule set they run lives in scantlings/verification-conformance and is discharged by the conformance command in RIGGING.md.

  @conformance
  Scenario: the implementation carries no standing perturbation
    Given the implementation sources
    When the verification-conformance rule set is run
    Then the "perturbation-quiescence" rule reports no match

  @conformance
  Scenario: every plank is a doc comment on a declaration
    Given the implementation sources
    When the verification-conformance rule set is run
    Then the "plank-form" rule reports no match

  @conformance
  Scenario: verification support declares no unmarked test double
    Given the verification support sources
    When the verification-conformance rule set is run
    Then the "forbidden-doubles" rule reports no match

  Rule: a scantling and its published URI are read over the network by consumers who never run this suite, so both are checked here. A scantling that declares a dialect must satisfy it: a mistyped keyword yields a schema that validates everything and an attestation that asserts nothing. Each count is named so an empty read fails rather than passes; three scantlings carry no dialect because they are proof contracts discharged by their own checkers.

  @conformance
  Scenario: every scantling declaring a dialect satisfies it
    Given the scantlings that declare a JSON Schema dialect
    When each is checked against the JSON Schema 2020-12 meta-schema
    Then all eight validate

  @conformance
  Scenario: every published schema URI names the packaged version
    Given the package version in "Cargo.toml"
    When the schema URIs in the scantlings and the example plans are read
    Then all thirteen name that version

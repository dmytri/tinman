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

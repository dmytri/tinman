Feature: scantling enumerations
  As a test author reading Tinman's published scantlings
  I want every value a scantling enumerates to be a value Tinman accepts
  So that a document my tooling validated is a document Tinman can parse

  Rule: the scantlings are published under a versioned URI and read by consumers who never run this suite, so a value a scantling declares is a promise. An enumeration that drifts from the production enumeration it constrains breaks that promise in one of two directions, and each direction is its own failure: a declared value Tinman rejects makes a valid document unusable, and an accepted value the scantling omits makes a working document fail validation. Neither direction is visible to a schema-conformance attestation, which only checks documents Tinman itself produced.

  @contract
  Scenario: every value a scantling enumeration declares is a value Tinman accepts
    Given the enumerations the scantlings declare
    When each declared value is parsed by the type its scantling describes
    Then every declared value is accepted
    And the enumerations read are not empty

  @contract
  Scenario: every variant a production enumeration accepts is declared by its scantling
    Given the production enumerations the scantlings constrain
    When each variant's serialized name is matched against its scantling enumeration
    Then every variant is a value its scantling declares

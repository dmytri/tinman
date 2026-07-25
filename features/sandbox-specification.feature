Feature: sandbox specification
  As a Tinman maintainer
  I want the record sandbox specification to hold a backend-neutral shape
  So that every backend reads the same portable specification

  @contract
  Scenario: the default record sandbox specification conforms to its schema
    Given the default sandbox specification for "tinman record"
    When the specification is serialized
    Then it conforms to the "sandbox-spec" schema in "scantlings/sandbox-spec.schema.json"

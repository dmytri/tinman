Feature: prepared process
  As a Tinman maintainer
  I want a prepared process to hold everything the PTY runner needs
  So that the PTY runner launches without knowing any backend

  @contract
  Scenario: a prepared process conforms to its schema
    Given a command specification for "printf hello"
    When the Bubblewrap backend prepares the process
    Then the prepared process conforms to the "prepared-process" schema in "scantlings/prepared-process.schema.json"

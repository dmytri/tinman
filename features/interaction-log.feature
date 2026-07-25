Feature: interaction log
  As a test author
  I want the recording written as a constrained YAML log
  So that I can read and edit the captured interaction

  @contract
  Scenario: the interaction log conforms to its schema
    Given a recording session for "opencode" with one key press and one snapshot
    When the session is written as a YAML interaction log
    Then the log conforms to the "interaction-log" schema in "scantlings/interaction-log.schema.json"

  Scenario: the interaction log records key presses and snapshots
    Given a recording session for "opencode"
    And the operator presses the key "h"
    And the operator takes a screen snapshot
    When the session is written as a YAML interaction log
    Then the log lists a key press "h"
    And the log lists one snapshot

@sandbox
Feature: driver protocol
  As a test author working in pytest, jest or bun test
  I want to drive Tinman from my own test runner
  So that I can puppeteer a terminal program in the language my suite is written in

  Rule: the driver exchanges one JSON message per line on stdin and stdout, so a client in any language needs only a subprocess and a JSON parser

  Scenario: the driver answers a launch request with a session identifier
    Given the Tinman driver is running
    When the test runner sends the request:
      """
      {"id": 1, "op": "launch", "command": "printf READY"}
      """
    Then the driver replies to request 1 with a session identifier

  Scenario: the driver rejects an unknown operation
    Given the Tinman driver is running
    When the test runner sends the request:
      """
      {"id": 2, "op": "teleport"}
      """
    Then the driver replies to request 2 with the error "unknown operation: teleport"

  Scenario: a failed action leaves the session usable
    Given the Tinman driver has a session running "printf READY"
    When the test runner requests the text "ABSENT" is present
    Then the driver replies with a failed result
    And the driver answers a later screen request for the same session

  Scenario: closing a session reclaims its sandbox resources
    Given the Tinman driver has a session running "printf READY"
    When the test runner closes the session
    Then the session's temporary sandbox directories no longer exist

  @contract
  Scenario: driver messages conform to the protocol schema
    Given the Tinman driver has a session running "printf READY"
    When the test runner requests the terminal object model
    Then every exchanged message conforms to the "driver-protocol" schema in "scantlings/driver-protocol.schema.json"

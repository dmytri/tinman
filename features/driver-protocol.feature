@sandbox
Feature: driver protocol
  As a test author working in pytest, jest or bun test
  I want to drive Tinman from my own test runner
  So that I can puppeteer a terminal program in the language my suite is written in

  Rule: the driver speaks JSON-RPC 2.0, one message per line on stdin and stdout. A test author reaching for Tinman from pytest, jest or bun test reaches for a JSON-RPC library that already exists, rather than for a wire format only Tinman speaks. The same framing carries the language servers, debug adapters and model-context servers already on their machine.

  Rule: a launch binds the system directories and whatever the session's sandbox spec names, and nothing more. Reaching a program outside that set is the `mounts` provision's job, per `scantlings/sandbox-spec.schema.json`. A program the sandbox cannot execute is a failed launch rather than a running session: answering with a session identifier for a process that never started leaves every later step asserting against a blank screen, and passing while it does so.

  Scenario: the driver answers a launch call with a session identifier
    Given the Tinman driver is running
    When the test runner sends the request:
      """
      {"jsonrpc": "2.0", "id": 1, "method": "launch", "params": {"command": "printf READY"}}
      """
    Then the driver replies to request 1 with a session identifier

  Scenario: a launch whose program the sandbox cannot reach fails
    Given the Tinman driver is running
    When the test runner sends the request:
      """
      {"jsonrpc": "2.0", "id": 3, "method": "launch", "params": {"command": "/nowhere/absent-program"}}
      """
    Then the driver replies to request 3 with a failed result
    And the failure names the program it could not start

  Scenario: an unknown method is answered with the reserved code
    Given the Tinman driver is running
    When the test runner sends the request:
      """
      {"jsonrpc": "2.0", "id": 2, "method": "teleport"}
      """
    Then the driver replies to request 2 with the error code -32601
    And the error data names the method "teleport"

  Scenario: a failed expectation is a result rather than an error
    Given the Tinman driver has a session running "printf READY"
    When the test runner requests the text "ABSENT" is present
    Then the driver replies with a result whose "ok" is false
    And the reply carries no error object

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

  Scenario: the driver exits when its stdin closes
    Given the Tinman driver is running
    When the test runner closes the driver's stdin
    Then the driver process exits with a success status
    And the driver leaves no session sandbox directory standing

  Scenario: a call missing a required parameter is answered with an invalid-params error
    Given the Tinman driver has a session running "printf READY"
    When the test runner sends the request:
      """
      {"jsonrpc": "2.0", "id": 2, "method": "capture", "params": {"within": "log", "role": "article"}}
      """
    Then the driver replies to request 2 with the error code -32602
    And the error data names the missing parameter "scope"
    And the driver answers a later screen request for the same session

  Scenario: a capture naming an unknown scope is answered with an invalid-params error
    Given the Tinman driver has a session running "printf READY"
    When the test runner sends the request:
      """
      {"jsonrpc": "2.0", "id": 2, "method": "capture", "params": {"within": "log", "role": "article", "scope": "evrything"}}
      """
    Then the driver replies to request 2 with the error code -32602
    And the error data names the rejected scope "evrything"

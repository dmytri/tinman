@sandbox
Feature: driver session
  As a test author driving a terminal program from my own test runner
  I want the same semantic verbs a browser driver offers
  So that a terminal test reads like the interaction a person would perform

  Background:
    Given the Tinman driver has a session running the fixture terminal program

  Rule: a terminal has no pointer. Where a browser driver clicks a coordinate, a terminal driver reaches its target the way a person does, by moving the selection and confirming it.

  Scenario: a launched program is sandboxed by default
    When the test runner requests the session's sandbox backend
    Then the reported backend is "bubblewrap"

  Rule: arriving somewhere and claiming something are different acts that shared one keyword. `expect` waited for text and asserted it in the same call, so a readiness check and a behavioural claim were indistinguishable in a plan, in the protocol and in the flow implementation, where both routed through the same wait. What that cost is legibility of failure: a barrier that never fires means the program did not reach the state, a claim that fails means the behaviour is wrong, and one keyword reported both identically. Readiness stays Tinman's problem because Tinman alone holds the PTY, the emulator and the frame timing; a caller polling over the protocol would reimplement that worse, and moving a hard problem does not solve it. Claims belong to whoever is testing, which is the caller's own suite wherever one exists.

  Scenario: a driver call can wait for readiness without claiming anything
    When the test runner waits for the text "READY"
    Then the driver reports the session reached that state

  Scenario: a wait that never arrives reports failure to arrive
    When the test runner waits for text the program never draws
    Then the driver reports the session did not reach that state
    And the failure names no failed expectation

  Rule: the driver answers a caller in another language over JSON-RPC 2.0, and that protocol carries an error object precisely so a failure can be described rather than inflicted. A seam that aborts instead hands the caller a closed pipe: no code, no message, nothing to act on, on the surface every external integration depends on. The contract forbids the constructs that abort; this scenario carries the half a prohibition cannot prove, which is that something useful is returned in their place.

  Scenario: a request the driver cannot serve is answered with an error
    When the test runner sends a request naming a method the driver does not serve
    Then the driver answers with a JSON-RPC error object
    And the session stays open for the next request

  Rule: the protocol scantling names every method the driver serves, and a contract scenario exchanging two of them attests two of them. That is how `wait` reached the driver while the scantling's enum did not carry it: a `wait` message would have failed the protocol it is served under, and the attestation stayed green because it never sent one. A method the enum names and no message exercises is a name nothing checks.

  @contract
  Scenario: every method the protocol names is served
    Given the methods named in "scantlings/driver-protocol.schema.json"
    When each is exchanged with the driver
    Then every exchange conforms to the protocol
    And the methods read are not empty

  @contract
  Scenario: the driver and assistant seams discharge the panic-free contract
    Given the implementation sources
    When the verifier checks the seams named in "scantlings/panic-free-seams.json"
    Then no counterexample is found

  Rule: what activation opened must be something the screen did not already show. The fixture draws its labelled input from the first frame, because the failure-report scenarios in `features/replay.feature` and `features/test-command.feature` assert on that text, so a `Then` naming it cannot tell an opened pane from an unopened one. These scenarios name the `Save` button instead, which `assets/examples/settings-flow.yaml` places inside Settings.

  Scenario: activating a menu item opens what it names
    When the test runner activates the "menuitem" named "Settings"
    Then the screen shows a "button" named "Save"

  Scenario: activation reaches an item the selection is not already on
    Given the menu's selected item is "Files"
    When the test runner activates the "menuitem" named "Settings"
    Then the screen shows a "button" named "Save"

  Scenario: activation leaves the selection on the item it named
    Given the menu's selected item is "Files"
    When the test runner activates the "menuitem" named "Settings"
    Then the selected "menuitem" is "Settings"

  Scenario: activation fails when the selection cannot reach the item
    Given the fixture program ignores directional keys
    When the test runner activates the "menuitem" named "Settings"
    Then the driver replies with a failed result
    And the failure reports the selection did not reach the "menuitem" named "Settings"

  Scenario: filling a textbox enters the value
    Given the test runner has activated the "menuitem" named "Settings"
    When the test runner fills the textbox labelled "Username" with "dmytri"
    Then the textbox labelled "Username" contains "dmytri"

  Scenario: pressing a key reaches the program
    When the test runner presses the key "q"
    Then the screen contains the text "Quit?"

  Scenario: an expectation on absent text fails with the screen contents
    When the test runner requests the text "Saved" is present
    Then the driver replies with a failed result
    And the failure reports the text was not found on screen

  Scenario: a locator that matches nothing names what it looked for
    When the test runner activates the "button" named "Deploy"
    Then the driver replies with a failed result
    And the failure reports no "button" named "Deploy" was found

  Scenario: a locator that matches several regions reports the ambiguity
    Given the fixture program shows two buttons named "OK"
    When the test runner activates the "button" named "OK"
    Then the driver replies with a failed result
    And the failure reports 2 matches for the "button" named "OK"

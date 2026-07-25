Feature: driver session
  As a test author driving a terminal program from my own test runner
  I want the same semantic verbs a browser driver offers
  So that a terminal test reads like the interaction a person would perform

  Background:
    Given the Tinman driver has a session running the fixture terminal program

  Scenario: a launched program is sandboxed by default
    When the test runner requests the session's sandbox backend
    Then the reported backend is "bubblewrap"

  Scenario: activating a menu item opens what it names
    When the test runner activates the "menuitem" named "Settings"
    Then the screen contains the text "Username"

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

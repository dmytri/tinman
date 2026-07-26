@sandbox
Feature: driver session
  As a test author driving a terminal program from my own test runner
  I want the same semantic verbs a browser driver offers
  So that a terminal test reads like the interaction a person would perform

  Rule: a terminal has no pointer. Where a browser driver clicks a coordinate, a terminal driver reaches its target the way a person does, by moving the selection and confirming it.

  Background:
    Given the Tinman driver has a session running the fixture terminal program

  Scenario: a launched program is sandboxed by default
    When the test runner requests the session's sandbox backend
    Then the reported backend is "bubblewrap"

  Scenario: activating a menu item opens what it names
    When the test runner activates the "menuitem" named "Settings"
    Then the screen contains the text "Username"

  Scenario: activation reaches an item the selection is not already on
    Given the menu's selected item is "Files"
    When the test runner activates the "menuitem" named "Settings"
    Then the screen contains the text "Username"

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

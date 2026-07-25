Feature: terminal object model locators
  As a test author
  I want to address the screen by role and by name
  So that a step keeps working when the program moves its panes

  Rule: locator resolution is mechanical. It reads the terminal object model built from the current screen and invokes no inference, so an authored test never needs a model.

  Scenario: a locator finds an item by role and name
    Given a terminal object model with a "menu" containing the menu items "Files", "Settings", and "Quit"
    When the locator for the "menuitem" named "Settings" is resolved
    Then the resolved region's text is "Settings"

  Scenario: a locator matches a name case-sensitively
    Given a terminal object model with a "menu" containing the menu items "Files", "Settings", and "Quit"
    When the locator for the "menuitem" named "settings" is resolved
    Then resolution fails and reports no match

  Scenario: a locator scoped to a named region searches only that region
    Given a terminal object model with a "list" named "Left" containing "README" and a "list" named "Right" containing "README"
    When the locator for the "listitem" named "README" is resolved within the region named "Right"
    Then the resolved region lies inside the region named "Right"

  Scenario: an unscoped locator matching two regions reports the ambiguity
    Given a terminal object model with a "list" named "Left" containing "README" and a "list" named "Right" containing "README"
    When the locator for the "listitem" named "README" is resolved
    Then resolution fails and reports 2 matches

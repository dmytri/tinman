Feature: terminal object model
  As a test author
  I want the virtual screen read as a tree of nested regions with semantic roles
  So that steps bind to what the program shows rather than to cell coordinates

  Rule: the terminal object model is the terminal's document object model. It is a semantic reading of the rendered screen, not a reconstruction of the program that drew it. Its geometry follows Ratatui: nested rectangles produced by horizontal and vertical splits.

  Scenario: a vertical split becomes two sibling regions
    Given a virtual screen 80 columns wide split vertically at column 40
    When the terminal object model is built
    Then the model's root has 2 child regions
    And the first child region covers columns 0 through 39

  Scenario: a bordered pane becomes a region carrying its title as its name
    Given a virtual screen showing a bordered pane titled "Files"
    When the terminal object model is built
    Then the model contains a region named "Files"

  Scenario: the lines of a bordered pane become list items
    Given a virtual screen showing a bordered pane titled "Files" listing "src", "tests", and "README"
    When the terminal object model is built
    Then the region named "Files" has the role "list"
    And that region has 3 child regions with the role "listitem"

  Scenario: a highlighted line is the selected item
    Given a virtual screen showing a bordered pane titled "Files" listing "src", "tests", and "README"
    And the line "tests" is rendered with reversed video
    When the terminal object model is built
    Then the selected item of the region named "Files" is "tests"

  Scenario: the bottom line becomes the status bar
    Given a virtual screen whose bottom line reads "NORMAL  main*  3 files"
    When the terminal object model is built
    Then the model contains a region with the role "statusbar"
    And that region's text is "NORMAL  main*  3 files"

  Scenario: a region keeps the screen cells it was built from
    Given a virtual screen showing a bordered pane titled "Files" listing "src" and "tests"
    When the terminal object model is built
    Then the region named "Files" reports the screen cell at its own row 1 column 1

  @contract
  Scenario: the terminal object model conforms to its schema
    Given a virtual screen showing a bordered pane titled "Files" listing "src" and "tests"
    When the terminal object model is serialized
    Then it conforms to the "tom" schema in "scantlings/tom.schema.json"

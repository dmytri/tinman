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
    Then the model contains a region with the role "status"
    And that region's text is "NORMAL  main*  3 files"

  Scenario: a region keeps the screen cells it was built from
    Given a virtual screen showing a bordered pane titled "Files" listing "src" and "tests"
    When the terminal object model is built
    Then the region named "Files" reports the screen cell at its own row 1 column 1

  Rule: the deterministic reading produces every role a written plan can address, because replay rebuilds the model with no model invocation. Inference enriches this model at capture time and is never a precondition for binding a locator a plan already carries.

  Rule: a menu is the role that carries a selection. Activating an item is moving that selection and confirming it, and a listener needs a menu announced as something they can act on, so the selection is what the role is for rather than an ornament on it. A line of words no selection distinguishes offers nothing to activate, and reading it as a menu names a control the screen does not carry: top draws its own summary across the top line and the model answered with menu items called "days," and "average:". Rendering one item in reverse video is what a terminal has to say a menu item is current, and it is available to the deterministic pass, so nothing needs inferring to tell a bar of controls from a sentence with gaps in it.

  Scenario: a menu bar becomes a menu of named menu items
    Given a virtual screen whose top line reads "  Files   Settings   Quit  "
    And the label "Files" is rendered with reversed video
    When the terminal object model is built
    Then the model contains a region with the role "menu"
    And that region has 3 child regions with the role "menuitem"
    And the second "menuitem" of that region is named "Settings"

  Scenario: a top line carrying no selection is not read as a menu
    Given a virtual screen whose top line reads "top - 18:52:59 up 9 days,  5:57,  0 user"
    When the terminal object model is built
    Then the model contains no region with the role "menu"

  Rule: a line that stops being a menu must still be something, or the correction trades a wrong role for a missing one and the screen's first line leaves the model entirely. A region nothing reads is worse than a region read wrongly, because a locator that binds the wrong role fails where a locator that binds nothing reports the screen is empty there. What top draws on that line is what an editor draws on its bottom one, the program's own report on itself, which is the status role the model already carries and already reads at the other edge of the screen.

  Scenario: a top line carrying no selection reads as a status region
    Given a virtual screen whose top line reads "top - 18:52:59 up 9 days,  5:57,  0 user"
    When the terminal object model is built
    Then the model contains a region with the role "status"
    And that region's text is "top - 18:52:59 up 9 days,  5:57,  0 user"

  Scenario: a bracketed label becomes a button
    Given a virtual screen showing "[ Save ]" at row 5 column 3
    When the terminal object model is built
    Then the model contains a region with the role "button" named "Save"

  Rule: a pane's bottom border carries text as often as its top border does, and the model reads only the top. Hints and status lines drawn into the lower border are a standing terminal idiom, and dropping them loses the part of a pane that says what to do next. It reads as a status region inside the pane it belongs to, the same role the screen's own bottom line takes, because that is what it is to a reader who cannot see it.

  Scenario: text in a pane's bottom border becomes a status region inside it
    Given a virtual screen showing a bordered pane titled "Ask Tinman" whose bottom border reads "enter to send | esc to leave"
    When the terminal object model is inferred
    Then the region named "Ask Tinman" contains a region with the role "status"
    And that status region shows "enter to send | esc to leave"

  Rule: a bordered pane holding the cursor is where typing goes, so it reads as a textbox rather than as a list of its own lines. The cursor is the only signal on a rendered screen that distinguishes a field being edited from a panel being displayed, and it is available to the deterministic pass, so no inference is needed to tell them apart. The distinction is not cosmetic: a reader who cannot see the screen is told to type into a list, and a plan addressing the region by role binds the wrong thing.

  Scenario: a bordered pane holding the cursor becomes a textbox
    Given a virtual screen showing a bordered pane titled "Ask Tinman" whose first line reads "record"
    And the cursor rests inside that pane
    When the terminal object model is inferred
    Then the region named "Ask Tinman" has the role "textbox"

  Scenario: a bordered pane without the cursor keeps its list reading
    Given a virtual screen showing a bordered pane titled "Files" listing "src" and "tests"
    And the cursor rests outside that pane
    When the terminal object model is inferred
    Then the region named "Files" has the role "list"

  Scenario: a labelled input field becomes a textbox
    Given a virtual screen showing "Username: ________" at row 3 column 1
    When the terminal object model is built
    Then the model contains a region with the role "textbox" labelled "Username"

  Scenario: a pane of blank-line separated entries becomes a log of articles
    Given a virtual screen showing a bordered pane titled "Output" holding the entries "build started" and "build finished" separated by a blank line
    When the terminal object model is built
    Then the region named "Output" has the role "log"
    And that region has 2 child regions with the role "article"

  Rule: colour is meaning on a terminal screen. An error line is red, a disabled item is dim, a selected row is reversed, and a test author asserting any of that is asking about the screen rather than about the program's internals. Tinman already reads an attributed cell grid and already derives roles from those attributes, so the attributes are in hand, and dropping them only moves every presentation assertion into a bespoke check against raw cells. This is the split a browser makes between the element tree and the computed style beside it.

  Scenario: a region drawn in a colour reports that colour
    Given a virtual screen whose bottom line reads "disk full" in red
    When the terminal object model is built
    Then the region with the role "status" is drawn in the foreground colour "red"

  Scenario: a region drawn plainly reports the terminal's own colours
    Given a virtual screen whose bottom line reads "ready" in no colour of its own
    When the terminal object model is built
    Then the region with the role "status" is drawn in the foreground colour "default"
    And that region is drawn in the background colour "default"

  Rule: a single computed style for a region drawn two ways would be a summary of the screen rather than a reading of it, and a test author asserting against a summary is asserting against Tinman rather than against their own program. An absence is the more honest answer, and a scenario reading it learns something true.

  Rule: the root region carries the `application` role, and WAI-ARIA requires a name on it, so the model must say what application this is. The program the operator asked about is the answer, and it is the answer a listener needs first: a screen reader announcing "application" and nothing else has told them the role and withheld the subject. Leaving it null is what the name-required constraint caught the moment it was written.

  Scenario: the root region is named for the program under inspection
    When the operator inspects the command "printf hi"
    Then the inspect output names the root region "printf hi"

  Rule: the emulator hands the builder nine cell attributes and the model carried two, so seven standard presentations were read off the wire and thrown away. That is the same omission the colour work closed, one layer over. The set is ECMA-48's, so `undercurl` stays out as a terminal extension rather than a standard: a model of what terminals do is worth more when it models what the standard says they do.

  Rule: these are semantic states, not decoration, which is why an accessibility layer needs them. A terminal says disabled with dim, says password field with hidden, and says done or deleted with strikeout. A reader that cannot report them is back to flat text for exactly the distinctions a listener most needs, which is the gap the model exists to close.

  Scenario Outline: a region drawn with a standard attribute reports it
    When the operator inspects a command that prints "marked" <attribute>
    Then the inspect output lists a region named "marked"
    And the inspect output reports that region is <attribute>

    Examples:
      | attribute         |
      | dim               |
      | italic            |
      | underlined        |
      | hidden            |
      | struck through    |
      | doubly underlined |

  Scenario: a region whose cells are drawn differently carries no style
    Given a virtual screen showing a bordered pane titled "Files" listing "src" and "tests"
    And the line "tests" is rendered in red while the rest of the pane is drawn plainly
    When the terminal object model is built
    Then the region named "Files" carries no style
    And the child region showing "tests" is drawn in the foreground colour "red"

  Rule: the cursor is what tells an operator the program is listening and where the next character will land, and it is already the signal the model uses to tell a field being edited from a panel being displayed. A program that hides it has said something observable, and a reported position nobody can see would be a fiction.

  Scenario: the model reports the cursor position
    Given a virtual screen showing "Username: ________" at row 3 column 1
    And the cursor rests at row 2 column 20
    When the terminal object model is built
    Then the model's cursor is at row 2 column 20

  Scenario: the model reports no cursor where the program has hidden it
    Given a virtual screen showing "Username: ________" at row 3 column 1
    And the program has hidden the cursor
    When the terminal object model is built
    Then the model carries no cursor

  @contract
  Scenario: the terminal object model conforms to its schema
    Given a virtual screen showing a bordered pane titled "Files" listing "src" and "tests"
    When the terminal object model is serialized
    Then it conforms to the "tom" schema in "scantlings/tom.schema.json"

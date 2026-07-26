Feature: virtual screen
  As a test author
  I want the terminal output of a launched program parsed into a virtual screen
  So that steps can assert what the program displayed

  Scenario: a launched program's output appears on the virtual screen
    Given a prepared process that runs "printf 'Hello, Tinman'"
    When the process is captured through a PTY
    Then the virtual screen contains the text "Hello, Tinman"

  Scenario: ANSI cursor positioning places text at the addressed cell
    Given a prepared process that writes "X" at row 3 column 5 using ANSI positioning
    When the process is captured through a PTY
    Then the virtual screen cell at row 3 column 5 shows "X"

  Rule: a full-screen program positions text by address rather than writing a line at a time, so the cells it skipped carry meaning. Every region boundary the terminal object model derives is measured in columns, which is why this layer is load-bearing rather than cosmetic.

  Scenario: text positioned with a gap keeps its column spacing
    Given a prepared process that writes "a" at row 1 column 1 and "b" at row 1 column 7
    When the process is captured through a PTY
    Then the virtual screen row 1 reads "a     b"

  Scenario: a wide character occupies both of its columns
    Given a prepared process that writes "你" at row 1 column 1
    When the process is captured through a PTY
    Then the virtual screen cell at row 1 column 1 shows "你"
    And the virtual screen cell at row 1 column 2 continues the character at column 1

  Scenario: reversed video covers every column of a wide character
    Given a prepared process that writes "ab你好cd" at row 1 column 1 in reversed video
    When the process is captured through a PTY
    Then every cell of row 1 from column 1 through column 8 is rendered with reversed video

  Scenario: a row highlighted by erasing to end of line reads as reversed throughout
    Given a prepared process that writes "Files" at row 1 column 1 in reversed video and erases to the end of the line
    When the process is captured through a PTY
    Then every cell of row 1 from column 1 through column 80 is rendered with reversed video

  Rule: a full-screen program addresses the cursor with several sequences, not one. An emulator that ignores a sequence draws the character at the cursor's current position instead, so the screen fills with text in the wrong cells and every region the terminal object model derives from it is measured wrong. The sequences below are emitted by the coding agents Tinman is built to drive.

  Rule: HPA below is the ECMA-48 horizontal position absolute, final byte backtick, and is a different sequence from CHA, final byte G. CHA and VPA are honoured already, so a step that reaches for CHA here would assert a behaviour the screen has and pin nothing.

  Scenario: a horizontal and vertical position sequence places text at the cell it names
    Given a prepared process that writes "X" at row 2 column 3 using the HVP sequence
    When the process is captured through a PTY
    Then the virtual screen cell at row 2 column 3 shows "X"

  Scenario: a horizontal position sequence places text at the column it names
    Given a prepared process that writes "X" at row 1 column 12 using the HPA sequence
    When the process is captured through a PTY
    Then the virtual screen cell at row 1 column 12 shows "X"

  Scenario: a repeat sequence draws the character it repeats
    Given a prepared process that writes "-" at row 1 column 1 and repeats it 9 times with the REP sequence
    When the process is captured through a PTY
    Then the virtual screen row 1 reads "----------"

  Scenario: with autowrap off the last column holds the final character written
    Given a prepared process that disables autowrap and writes 85 characters ending in "Z" at row 1 column 1
    When the process is captured through a PTY
    Then the virtual screen cell at row 1 column 80 shows "Z"

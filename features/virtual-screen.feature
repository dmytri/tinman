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

  Rule: a row reads as the columns hold it. A full-screen program positions text by address and leaves the cells between untouched, so the blanks it skipped are part of what it displayed. A reading that closes those gaps reports a screen the program never drew, and every region boundary derived from it lands in the wrong column.

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

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

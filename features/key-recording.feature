Feature: key recording
  As a test author
  I want key presses recorded and forwarded while I drive the program
  So that the recording reproduces exactly what I typed

  Scenario: key presses are recorded in order
    Given a recording session
    When the operator presses the keys "h", "i", and Enter
    Then the session's recorded key events are "h", "i", "Enter" in that order

  Scenario: recorded keys are forwarded to the launched program
    Given a prepared process that runs "cat"
    And the process is captured through a PTY
    When the operator presses the keys "h", "i", and Enter
    Then the virtual screen contains the text "hi"

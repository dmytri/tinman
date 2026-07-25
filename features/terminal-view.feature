Feature: terminal view
  As an operator
  I want the captured screen rendered in the Ratatui view
  So that I can watch the program while I record

  Scenario: the capture view renders the virtual screen
    Given a virtual screen that shows "PROMPT>"
    When the capture view is rendered to a 40 by 10 test terminal
    Then the rendered frame contains "PROMPT>"

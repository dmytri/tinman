Feature: command invocation
  As a test author
  I want "tinman record" to capture the target command and its arguments
  So that Tinman knows which program to launch under a PTY

  Scenario: record captures the target command and its arguments
    When the operator runs "tinman record opencode --model deepseek"
    Then the capture target program is "opencode"
    And the capture target arguments are "--model" and "deepseek"

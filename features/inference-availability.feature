Feature: inference availability
  As an operator
  I want Tinman to work whether or not inference is configured
  So that a missing credential degrades instead of failing

  Rule: inference unavailable is a normal degraded mode, not an error.

  Scenario: a dotenv file supplies the provider credential
    Given a working directory holding a ".env" file that sets "TINMAN_API_KEY" to "sk-file-key"
    And the environment does not set "TINMAN_API_KEY"
    When Tinman resolves its inference credential
    Then the resolved credential is "sk-file-key"

  Scenario: the environment overrides the dotenv file
    Given a working directory holding a ".env" file that sets "TINMAN_API_KEY" to "sk-file-key"
    And the environment sets "TINMAN_API_KEY" to "sk-env-key"
    When Tinman resolves its inference credential
    Then the resolved credential is "sk-env-key"

  Scenario: inference is unavailable without a credential
    Given neither the environment nor a dotenv file sets "TINMAN_API_KEY"
    When Tinman checks whether inference is available
    Then inference is reported unavailable

  Scenario: degraded help fills the tagline with the unavailable notice
    Given inference is unavailable
    When the operator runs "tinman --help" in an interactive terminal
    Then the tagline line is the body of the asset at "assets/help/inference-unavailable.txt"

  Scenario: degraded help omits the assistant prompt
    Given inference is unavailable
    When the operator runs "tinman --help" in an interactive terminal
    Then the help output omits the body of the asset at "assets/help/assistant-prompt.txt"

  Scenario: degraded help exits successfully
    Given inference is unavailable
    When the operator runs "tinman --help" in an interactive terminal
    Then the command exits with status 0

  Scenario: replay runs without a credential
    Given a harness plan driving the fixture terminal program
    And neither the environment nor a dotenv file sets "TINMAN_API_KEY"
    When that plan is replayed
    Then the replay passes

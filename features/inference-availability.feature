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

  Rule: the absence of the box is asserted against the terminal object model, for the same reason its presence is. A bordered region's text never appears contiguously in the bytes, since border characters sit between the title and the body, so a search of the raw output for the prompt asset's body cannot find it whether the box was drawn or not. An absence assertion that cannot fail is worse than none: it reports the degraded path guarded while the guard is gone.

  Scenario: degraded help omits the assistant prompt
    Given inference is unavailable
    When the operator runs "tinman --help" in an interactive terminal
    Then no bordered region titled "Ask Tinman" is drawn

  Rule: the tagline is decoration, so it waits on nothing an operator can notice. It is the one inference call on the path of a command an operator runs to read documentation, and the help text is complete without it, so its ceiling is short and the line is simply dropped when the provider does not answer inside it. A ceiling sized for a generation call would hold the most-run command for tens of seconds to fill one cosmetic line.

  Scenario: a slow provider does not delay the help text
    Given the inference credential is configured
    And the inference provider endpoint accepts the connection and never answers
    When the operator runs "tinman --help" with stdout redirected to a file
    Then the help output is the asset at "assets/help/tinman.txt" with the tagline line removed
    And the command completed within 10 seconds

  Scenario: degraded help exits successfully
    Given inference is unavailable
    When the operator runs "tinman --help" in an interactive terminal
    Then the command exits with status 0

  Scenario: replay runs without a credential
    Given a harness plan driving the fixture terminal program
    And neither the environment nor a dotenv file sets "TINMAN_API_KEY"
    When that plan is replayed
    Then the replay passes

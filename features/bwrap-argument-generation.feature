Feature: bubblewrap argument generation
  As a Tinman maintainer
  I want generated Bubblewrap arguments to enforce isolation
  So that the operator's home, environment and network stay hidden

  @contract
  Scenario: generated Bubblewrap arguments satisfy the isolation policy
    Given a command specification for "opencode"
    And a sandbox specification that denies network access
    When the Bubblewrap backend generates its arguments
    Then the arguments satisfy the isolation policy in "scantlings/bwrap-isolation-policy.json"

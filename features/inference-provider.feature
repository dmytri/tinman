Feature: inference provider
  As a Tinman maintainer
  I want every inference use to go through one provider interface
  So that a second provider is added without touching the callers

  Rule: OpenRouter is the configured provider. The interface is the seam, so a different provider changes configuration rather than code.

  Scenario: an unreachable provider reports inference unavailable
    Given the inference credential is configured
    And the inference provider endpoint is unreachable
    When Tinman checks whether inference is available
    Then inference is reported unavailable

  Scenario: a rejected credential reports inference unavailable
    Given the inference provider rejects the configured credential
    When Tinman checks whether inference is available
    Then inference is reported unavailable

  @inference
  Scenario: the configured provider answers a completion request
    Given the inference credential is configured
    When the assistant request "reply with the single word READY" is sent
    Then the provider's reply contains "READY"

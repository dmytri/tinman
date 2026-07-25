Feature: inference provider
  As an operator
  I want to point Tinman at any OpenAI-compatible provider
  So that the default choice of OpenRouter is never a lock-in

  Rule: Tinman speaks the OpenAI-compatible chat-completions protocol. OpenRouter is the default endpoint, not a requirement. The credential, the endpoint and the model are configuration, read from the environment or a dotenv file, so reaching a different compatible provider changes configuration rather than code.

  Scenario: an unconfigured provider is OpenRouter
    Given neither the environment nor a dotenv file sets "TINMAN_BASE_URL" or "TINMAN_MODEL"
    When an inference request is built
    Then the request addresses "https://openrouter.ai/api/v1" with the model "deepseek/deepseek-v4-flash"

  Scenario: a configured provider replaces the default
    Given the environment sets "TINMAN_BASE_URL" to "https://api.example-llm.test/v1" and "TINMAN_MODEL" to "meta-llama/llama-4-70b"
    When an inference request is built
    Then the request addresses "https://api.example-llm.test/v1" with the model "meta-llama/llama-4-70b"

  @contract
  Scenario: a built request satisfies the provider contract
    Given the inference credential is configured
    When an inference request is built
    Then it conforms to the "inference-request" schema in "scantlings/inference-request.schema.json"

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

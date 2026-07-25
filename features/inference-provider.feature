Feature: inference provider
  As an operator
  I want to point Tinman at any OpenAI-compatible provider
  So that the default choice of OpenRouter is never a lock-in

  Rule: Tinman speaks the OpenAI-compatible chat-completions protocol. OpenRouter is the default endpoint, not a requirement. The credential, the endpoint and the model are configuration, read from the environment or a dotenv file, so reaching a different compatible provider changes configuration rather than code.

  Scenario: an unset endpoint defaults to OpenRouter
    Given neither the environment nor a dotenv file sets "TINMAN_BASE_URL"
    When an inference request is built
    Then the request addresses "https://openrouter.ai/api/v1"

  Scenario: a configured endpoint addresses a different provider
    Given the environment sets "TINMAN_BASE_URL" to "https://api.example-llm.test/v1"
    When an inference request is built
    Then the request addresses "https://api.example-llm.test/v1"

  Scenario: an unset model defaults to the bundled model
    Given neither the environment nor a dotenv file sets "TINMAN_MODEL"
    When an inference request is built
    Then the request names the model "deepseek/deepseek-v4-flash"

  Scenario: a configured model overrides the default
    Given the environment sets "TINMAN_MODEL" to "meta-llama/llama-4-70b"
    When an inference request is built
    Then the request names the model "meta-llama/llama-4-70b"

  Scenario: the credential is sent as a bearer token
    Given the environment sets "TINMAN_API_KEY" to "sk-test-key"
    When an inference request is built
    Then the request carries the authorization header "Bearer sk-test-key"

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

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

  Rule: a provider that refuses a connection and a provider that accepts one and then withholds its answer are different faults with the same remedy. The first fails on its own, so a caller needs no ceiling to survive it. The second never returns, so only a ceiling ends it, and a request built without one waits forever on a socket that stays open. Tinman calls this path from `tinman --help`, so an operator with a stalled provider gets a hung command rather than a report.

  Rule: an availability probe and a generation call are two operations, and one ceiling cannot serve both. A probe asks only whether the provider answers, so it is bounded tightly and a slow answer is indistinguishable from no answer. A generation call asks a model to produce a structured document, which legitimately takes tens of seconds, so a ceiling sized for the probe truncates real work and reports absence where the provider was answering correctly. The two bounds are pinned from opposite directions: the stalled-provider scenario below is the ceiling, and the @inference scenarios that assert a real model produced a real result are the floor. A single shared ceiling satisfies whichever of the two was written most recently and silently breaks the other.

  Scenario: a stalled provider reports inference unavailable within a bounded time
    Given the inference credential is configured
    And the inference provider endpoint accepts the connection and never answers
    When Tinman checks whether inference is available
    Then the stalled endpoint received the request
    And inference is reported unavailable within 30 seconds

  @inference
  Scenario: the configured provider answers a completion request
    Given the inference credential is configured
    When the assistant request "reply with the single word READY" is sent
    Then a reply is parsed from the provider's response
    And the parsed reply carries non-empty content

  Scenario: a built request carries the configured credential as a bearer token
    Given the environment sets "TINMAN_API_KEY" to "sk-or-v1-8f3c2a91"
    When an inference request is built
    Then the request carries the authorization header "Bearer sk-or-v1-8f3c2a91"

  Scenario: a request built without a credential carries no authorization header
    Given neither the environment nor a dotenv file sets "TINMAN_API_KEY"
    When an inference request is built
    Then the request carries no authorization header

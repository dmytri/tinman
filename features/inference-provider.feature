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

  Rule: each timed seam carries a wall-clock ceiling, and a ceiling nobody measures is a guess that fails silently, because the seam it bounds degrades rather than erroring. One contract lists them and one scenario attests them all, so a new timed seam adds a line to the contract rather than a scenario to this file.

  @contract
  Scenario: every timed seam abandons its wait at its ceiling
    Given the seams and ceilings in "scantlings/latency-budgets.json"
    And a dependency that never answers
    When each seam is exercised and timed
    Then every seam gave up at its ceiling
    And the seams read are not empty

  Rule: a reasoning model spends its thinking budget on whatever it is asked, and the tagline asks for six words. Measured against the configured provider, the request as first written returned nothing inside forty seconds, so the tagline never filled and the help simply rendered without it, which reads as a design choice rather than a failure. With reasoning disabled the same request answers in one to two seconds. The lever is the request, not the ceiling: raising a budget to fit a call that should never have been slow hides the cause and pays the latency for ever.

  Scenario: the tagline request asks the provider not to reason
    Given the inference credential is configured
    When the acronym request is built
    Then the request disables reasoning

  Rule: asking a model for an acronym gets a claim that the words spell the name, and this project checks claims rather than repeating them. A deterministic pass walks the expansion for the letters of "tinman" in order and raises them, which both proves the acronym and shows the reader where it hides. It is the same rule the naming pass already follows, where the model proposes and the screen decides; here the expansion decides. Where the letters do not appear in order the expansion is not an acronym at all, so it is replaced rather than dressed up.

  Rule: replacement is bounded by the same tagline ceiling as the first attempt. An unbounded retry would spin for ever on a model that keeps missing, which is the end-state failure the help's spinner rule forbids, so the ceiling governs the whole attempt rather than each try.

  Scenario: the tagline raises the letters that spell Tinman
    Given the provider answers "terminal interaction and networkless model-driven application navigator"
    When the tagline is generated
    Then the tagline reads "Terminal Interaction and Networkless Model-driven Application Navigator"

  Scenario: the raised letters begin words where the expansion allows
    Given the provider answers "terminal interaction and networkless model-driven application navigator"
    When the tagline is generated
    Then every raised letter begins a word

  Scenario: an expansion that does not spell Tinman is asked again
    Given the provider answers "a testing tool for terminals"
    When the tagline is generated
    Then that expansion is not on the tagline line

  Scenario: a provider that keeps missing settles inside the ceiling
    Given the provider always answers "a testing tool for terminals"
    When the tagline is generated
    Then the tagline line settles inside the tagline ceiling

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

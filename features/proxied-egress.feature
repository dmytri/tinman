Feature: proxied egress
  As someone measuring a real coding agent
  I want the agent's network traffic to cross a proxy Tinman runs
  So that what the agent sent is ground truth read off the wire rather than scraped from what it chose to print

  Background:
    Given a recorded upstream that answers on a real port

  Rule: the sandbox denies network, so Tinman already stands between the target and its provider. Routing egress through a proxy Tinman runs turns that position into an instrument: tokens per request, turns as request count, latency per call, and whether a given text actually reached the model's context all become facts read off the wire. Scraping the same facts from each agent's own output would compare three agents through three sources of unequal quality, which is not a comparison.

  Rule: the proxy is adopted rather than written. hudsucker is a maintained MITM HTTP and HTTPS proxy under MIT or Apache-2.0, and har is a HAR serialization library under MIT; both were read from the registry on 2026-07-31 rather than recalled. What stays ours is the part no crate carries: the policy that decides which host is reachable, where the credential lives, and which fault is injected.

  Rule: HAR is the cassette format because it is the format browser tooling already records and replays, and a standard beats a bespoke one. The trap it brings is strict matching: any value the subject varies per run, such as a session identifier or a timestamp, makes a recorded entry unmatchable. A HAR file is editable JSON, which is the same answer the browser tooling gives.

  Rule: fault injection is the browser devtools model, and it earns its place by reaching the conditions nobody can test today. A provider's rate limit and its server errors are exactly the responses an agent's retry logic is written for and never exercised against, because asking a real provider for a 429 on demand is not something a real provider offers.

  Rule: replay is proved against a running upstream that answers differently, never against a stopped one. Stopping the subject first makes the closing assertion unfalsifiable: a stopped upstream cannot receive a request whatever the proxy does, so the step reports a pass it could not have withheld. Leaving it running and serving a different body turns the same question into one a run decides, and turns an absence into a positive carrier, because a reply that reached the network arrives carrying the wrong body rather than merely failing to be absent.

  @sandbox
  Scenario: a sandboxed target reaches an allowed host only through the proxy
    Given a plan whose sandbox sets network to "proxy"
    When the target requests the allowed host directly by address
    Then the direct request is refused
    And the same request through the proxy is answered

  @sandbox
  Scenario: the credential never enters the sandbox
    Given a plan whose sandbox sets network to "proxy" and an inference credential held by Tinman
    When the target prints its whole environment and then calls the provider
    Then the printed environment carries no inference credential
    And the request the upstream received carries the credential

  Scenario: the exchange is recorded as a HAR entry
    Given the proxy is recording to a HAR file
    When a request crosses the proxy and is answered
    Then the HAR file carries one entry for that request
    And the entry records the request method, the request URL and the response status

  Scenario: replay answers from the recording without reaching the network
    Given a HAR file carrying one recorded exchange
    And the upstream now answers that same request with a different body
    When the same request is made
    Then the recorded body is served rather than the upstream's current one
    And the upstream received no request

  Scenario: a request no recorded entry matches fails rather than reaching the network
    Given a HAR file carrying one recorded exchange
    When a request the recording does not carry is made
    Then the request fails naming the entry it did not match
    And the upstream received no request

  Scenario: an injected status is served instead of the upstream response
    Given the proxy is injecting the status 429
    When a request crosses the proxy
    Then the response carries the status 429
    And the upstream received no request

  Scenario: injected latency delays the response by the configured duration
    Given the proxy is injecting a latency of 2 seconds
    When a request crosses the proxy
    Then the response arrives no sooner than 2 seconds after the request
    And the response carries the upstream's own body

  Scenario: the offline fault refuses the connection
    Given the proxy is injecting the offline fault
    When a request crosses the proxy
    Then the connection is refused
    And the upstream received no request

  @contract
  Scenario: a sandbox specification routing egress through the proxy satisfies its scantling
    Given a sandbox specification setting network to "proxy"
    When the specification is validated
    Then the specification conforms to the "sandbox-spec" schema in "scantlings/sandbox-spec.schema.json"

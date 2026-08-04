Feature: inference profiles
  As a Tinman maintainer
  I want each inference use to carry its own sampling profile
  So that the acronym stays playful while the assistant stays correct

  Rule: the acronym is cosmetic and optimizes for novelty. The assistant proposes commands a user may run, so it optimizes for correctness, determinism and instruction following.

  Scenario: the acronym profile samples at a very high temperature
    When the acronym request is built
    Then the request temperature is 1.4

  Scenario: the assistant profile samples at a low temperature
    When the assistant request is built
    Then the request temperature is 0.1

  Scenario: both profiles address the same configured model
    Given the configured model is "deepseek/deepseek-v4-flash-0731"
    When the acronym request and the assistant request are built
    Then both requests name the model "deepseek/deepseek-v4-flash-0731"

Feature: semantic capture
  As a test author asserting on a coding agent's output
  I want to collect every item in a scrolling pane
  So that I can read the whole conversation instead of one screenful

  Rule: capture is mechanical at run time. The runtime scrolls the pane, collects its items, deduplicates them and returns structured data. Inference decides how a pane is captured; it never runs while a test does.

  Background:
    Given the Tinman driver has a session running the fixture terminal program
    And the fixture program shows a "message-pane" holding 12 messages in a 5 line window

  Scenario: capturing every item collects past the visible window
    When the test runner captures every "message" in the "message-pane" as "conversation"
    Then the capture named "conversation" holds 12 items

  Scenario: captured items keep their screen order
    When the test runner captures every "message" in the "message-pane" as "conversation"
    Then the first item of the capture named "conversation" is "message 1"
    And the last item of the capture named "conversation" is "message 12"

  Scenario: items repeated across scroll positions are collected once
    Given the fixture program repeats its last 2 messages at each scroll position
    When the test runner captures every "message" in the "message-pane" as "conversation"
    Then the capture named "conversation" holds 12 items

  Scenario: capturing the visible scope reads only the current window
    When the test runner captures the visible "message" items in the "message-pane" as "window"
    Then the capture named "window" holds 5 items

  Scenario: capturing a pane that never stops scrolling fails within its budget
    Given the fixture program scrolls its "message-pane" without ever reaching an end
    When the test runner captures every "message" in the "message-pane" as "conversation"
    Then the driver replies with a failed result
    And the failure reports the capture reached its scroll limit

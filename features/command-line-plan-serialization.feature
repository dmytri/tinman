Feature: command-line plan serialization
  As an operator who has finished exploring
  I want the session written out as an ordinary plan
  So that what I just did becomes a test I commit rather than scrollback I lose

  Rule: exploration that cannot be serialized is a demonstration rather than a probe, so this is the property the command line exists to satisfy. The written plan is a plan like any other: a person reads it, edits it, commits it, and runs it with test. It is not a transcript, not a log, and carries no record of having been arrived at interactively.

  Rule: the plan is written from the steps that held. A step that failed describes a screen that never was, and carrying it would hand the operator a permanently red test they did nothing to earn. This is the documented-examples rule, which the record and inspect surfaces already obey, reaching the third surface that writes a plan.

  @sandbox
  Scenario: the written plan carries every step the session performed
    Given a session named "work" running "top"
    And the operator has expected "PID" and pressed "q" in that session
    When the operator executes "tinman close --session work --output probe.yaml"
    Then "probe.yaml" is a plan whose steps are the expectation and the keypress in that order

  @sandbox
  Scenario: a step that failed is left out of the written plan
    Given a session named "work" running "top"
    And the operator has expected "PID" and then expected "NOSUCH" in that session
    When the operator executes "tinman close --session work --output probe.yaml"
    Then "probe.yaml" is a plan whose only step is the expectation naming "PID"

  Rule: replaying the written plan is the only check that catches a reframing, because a plan can carry every step and still describe a different run. A step written against a screen the replay never reaches passes review and fails execution, and the difference is invisible to a reader comparing two lists of steps. So the property is stated as a round trip rather than as a comparison.

  @sandbox
  Scenario: the written plan replays against the same program
    Given a session named "work" running "top"
    And the operator has expected "PID" in that session
    When the operator executes "tinman close --session work --output probe.yaml"
    And the operator executes "tinman test probe.yaml"
    Then the command exits with a zero status
    And the replay ran the expectation the session performed

  Rule: a session name is how the command line addresses a running program and means nothing to a plan, which drives one program per tui block and needs no handle for it. A plan carrying the name would be a reframing of exactly the kind this feature exists to prevent: readable, plausible, and describing a concept the plan language does not have.

  @sandbox
  Scenario: the written plan carries no session name
    Given a session named "work" running "top"
    And the operator has expected "PID" in that session
    When the operator executes "tinman close --session work --output probe.yaml"
    Then "probe.yaml" names the program "top" and carries no session name

  Rule: a plan that omits its isolation adopts whatever the reader's defaults are at replay time, so it describes a run that may differ from the one it came from. Writing the sandbox out makes the plan readable on its own terms and keeps it truthful if a default ever moves.

  Rule: isolation is not selectable from the command line today, so what this pins is that the block is written at all rather than which policy it names. Stated plainly because the scenario reads like it discriminates between policies and does not: the earlier form asserted the network was denied while denial was the only reachable state, so it would have held whether serialization wrote the session's isolation or a hardcoded default. When launching gains an isolation option, this is where the policy assertion belongs.

  @sandbox
  Scenario: the written plan states its isolation rather than leaving it implied
    Given a session named "work" running "top"
    And the operator has expected "PID" in that session
    When the operator executes "tinman close --session work --output probe.yaml"
    Then "probe.yaml" declares the sandbox the session ran under

  Rule: a one-shot command is a session of one step, so it writes the same plan a named session would have written for that step. Two spellings producing two plan shapes would make the written artifact depend on how the operator happened to invoke the tool, which is the reframing this feature forbids.

  @sandbox
  Scenario: a one-shot command writes the plan its single step describes
    When the operator executes "tinman expect PID top --output probe.yaml"
    Then "probe.yaml" is a plan whose only step is the expectation naming "PID"

  @contract
  Scenario: a plan written from a session satisfies the plan schema
    Given a plan written from a session that expected "PID" against "top"
    When the plan is validated
    Then it conforms to the "harness-plan" schema in "scantlings/harness-plan.schema.json"

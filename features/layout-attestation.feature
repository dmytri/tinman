Feature: layout attestation
  As an operator who knows what a program's screen should look like
  I want to attest the whole model against a layout I wrote down
  So that a program drawing the right things in the wrong places fails

  Rule: a layout is a constraint on the model rather than a list of spot checks. Naming a text present somewhere on screen says nothing about where it sits or what it belongs to, and a program that moved a column would satisfy every such check. A scantling names the regions that must exist, their place, and the presentation they must respect, which is exhaustive where a list of texts is a sample.

  Rule: the model already carries what a layout needs, so this route asks nothing new of it. Every region carries a rect of x, y, width and height, and children are ordered, so a position is a property to constrain and an ordinal is an index. That is why attesting reaches a table cell that a locator cannot: a locator addresses by role and name, and a cell's identity comes from the column above it, which the model relates only by position.

  Rule: the form is JSON Schema, which this project already uses for every other scantling and already points at its own interface. A survey on 2026-08-02 found no maintained alternative: jsonschema published that day against 77.1 million downloads, where boon last published 2025-01-07, valico 2023-05-13, and jsonschema-valid 2023-11-08.

  @sandbox
  Scenario: a model satisfying its layout scantling exits zero
    Given a layout scantling at "top-layout.json" requiring a table whose column headers name "PID" and "%CPU"
    When the operator executes "tinman expect --conforms top-layout.json top"
    Then the command exits with a zero status

  @sandbox
  Scenario: a model missing a region its layout requires names what departed
    Given a layout scantling at "top-layout.json" requiring a region named "Footer"
    When the operator executes "tinman expect --conforms top-layout.json top"
    Then the command exits with a non-zero status
    And the failure names "Footer" as the region the model lacked

  Rule: drawing the right thing in the wrong place is the fault this exists to catch, and it is the one a text search cannot see. A column header that moved is still on screen, so every expectation naming it still holds, while a person reading the program sees a broken table.

  @sandbox
  Scenario: a region drawn outside the bounds its layout gives fails on its position
    Given a layout scantling at "top-layout.json" requiring the "%CPU" column header to start beyond column 200
    When the operator executes "tinman expect --conforms top-layout.json top"
    Then the command exits with a non-zero status
    And the failure reports the position the region was drawn at

  Rule: a scantling that constrains nothing passes every model, which is an attestation that cannot fail. This project has shipped that shape before, as a scantling that was valid JSON and an invalid schema, and as an optional property read as though it were required. What a run can decide is that the document is a schema and that it carries a constraint at all; whether its constraints are the right ones stays a reader's judgment, and saying so is better than a check that implies otherwise.

  Rule: a scantling is read before the program is launched, so a broken one is refused without spending a sandbox on it. Launching first would make the operator wait on a real process to be told their schema has a typo, and it would put a check that needs no external tool into the tier that needs one. That the program is never launched is asserted rather than assumed, because it is the whole reason these two sit in the default tier.

  Scenario: a scantling that is not a valid schema is refused rather than satisfied
    Given a layout scantling at "broken.json" that is valid JSON and not a valid schema
    When the operator executes "tinman expect --conforms broken.json top"
    Then the command exits with a non-zero status
    And the failure reports the scantling is not a schema
    And the program was never launched

  Scenario: a scantling carrying no constraint is refused rather than satisfied
    Given a layout scantling at "empty.json" that constrains nothing
    When the operator executes "tinman expect --conforms empty.json top"
    Then the command exits with a non-zero status
    And the failure reports the scantling constrains nothing
    And the program was never launched

  Rule: the second half of the operator's question is a layout after a sequence, so attesting composes with the option that runs a plan first. Nothing new is owed for it beyond the composition holding.

  @sandbox
  Scenario: a layout is attested after a plan's steps have run
    Given a plan at "reach.yaml" whose steps press "q" against the fixture terminal program
    And a layout scantling at "quit-layout.json" requiring the region the keypress draws
    When the operator attests "quit-layout.json" after "reach.yaml" against the fixture terminal program
    Then the command exits with a zero status

  Rule: an attestation that held is a plan step like any other, so the plan language carries it and a probe that attested serializes without a gap. A command able to write a plan the schema cannot express would break the property the written plan exists to satisfy.

  @sandbox
  Scenario: the written plan carries the attestation it proved
    Given a layout scantling at "top-layout.json" requiring a table whose column headers name "PID" and "%CPU"
    When the operator executes "tinman expect --conforms top-layout.json top --output probe.yaml"
    Then the command exits with a zero status
    And "probe.yaml" is a plan whose only step attests "top-layout.json"

  Rule: a committed plan reaches attestation through replay where the command line reaches it through expect, so one seam has two callers and only one of them was exercised. A written plan that cannot be replayed is the serialization property failing at the one step it was extended for, and a replay arm nobody runs is the shape this project keeps finding: planked, joined, and proven by nothing.

  @sandbox
  Scenario: a plan carrying an attestation is attested at replay
    Given a plan at "layout.yaml" whose attestation names a layout "top" satisfies
    When the operator executes "tinman test layout.yaml"
    Then the command exits with a zero status

  Rule: the failing direction is the one that carries the proof. An attestation step that silently did nothing at replay would exit zero against any layout, so the passing scenario alone would report the arm working while it ran nothing at all.

  @sandbox
  Scenario: a plan whose attestation departs from the drawn model fails at replay
    Given a plan at "layout.yaml" whose attestation names a region "top" does not draw
    When the operator executes "tinman test layout.yaml"
    Then the command exits with a non-zero status
    And the failure names the region the model lacked

  @contract
  Scenario: a plan carrying an attestation step validates
    Given the plan fixture carrying a step that attests a layout scantling
    When the plan is validated
    Then it conforms to the "harness-plan" schema in "scantlings/harness-plan.schema.json"

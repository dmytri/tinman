Feature: methodology conformance
  As a Tinman maintainer
  I want methodology breaches to surface as failing verification
  So that a violation is discovered by a run rather than by eye

  Rule: these scenarios attest the project's own method, not the product. They leave with the method they guard.

  Rule: the rule set is one scantling discharged by one command, so a new methodology check lands as a rule entry rather than as a new scenario. The rule set lives in scantlings/verification-conformance and is discharged by the conformance command in RIGGING.md. One invocation scans every rule, so one attestation covers the set: a scenario per rule restates the same exhaustive scan and grows the feature by one scenario for every rule added. The named floor keeps an empty or misconfigured rule set from reporting no match because it carries no rules.

  @conformance
  Scenario: the verification-conformance rule set reports no match
    Given the implementation sources and the verification support sources
    When the verification-conformance rule set is run
    Then no rule in the set reports a match
    And the rule set carries at least the plank-form, plank-presence, perturbation-quiescence and forbidden-doubles rules

  Rule: a scantling and its published URI are read over the network by consumers who never run this suite, so both are checked here. A scantling that declares a dialect must satisfy it: a mistyped keyword yields a schema that validates everything and an attestation that asserts nothing. Each count is named so an empty read fails rather than passes; four scantlings carry no dialect because they are proof contracts discharged by their own checkers.

  @conformance
  Scenario: every scantling declaring a dialect satisfies it
    Given the scantlings that declare a JSON Schema dialect
    When each is checked against the JSON Schema 2020-12 meta-schema
    Then all nine validate

  @conformance
  Scenario: every published schema URI names the packaged version
    Given the package version in "Cargo.toml"
    When the schema URIs in the scantlings and the example plans are read
    Then all fourteen name that version

  Rule: the plank joins below need two sources joined, the plank inventory and the step-usage pattern set, so their logic lives in step definitions rather than in an ast-grep rule. The pattern set comes from the derived step-usage command, which reports each step-definition pattern literal untruncated; the join is exact string membership, with no normalization on either side.

  @conformance
  Scenario: every plank names a current step-definition pattern
    Given the plank inventory and the step-usage pattern set
    When each plank string is matched against the pattern set
    Then every plank string is a pattern the step definitions declare
    And the plank inventory is not empty

  @conformance
  Scenario: every step definition binds at least one scenario
    Given the step-usage pattern set and the scenarios in the specs
    When each pattern is matched against the steps the scenarios carry
    Then every pattern binds at least one scenario
    And the pattern set is not empty

  Rule: a provisional plank names a scenario rather than a pattern, because the skeleton it marks has no step definition yet. It is spent the moment Captain disposes of that skeleton: a promoted scenario no longer carries @captain and owes its seam a real plank, and a discarded scenario is gone entirely and owes the annotation's removal. Both spent states read as ordinary coverage until someone runs the inventory by hand, which is how five spent annotations survived a full harbour. An empty provisional set is the healthy resting state, so the floor here guards the reader rather than the count: an unreadable inventory and a genuinely empty one both report zero.

  @conformance
  Scenario: every provisional plank names a scenario still awaiting review
    Given the provisional plank references and the scenarios in the specs
    When each reference is matched against the scenarios the specs still tag "@captain"
    Then every provisional plank names a scenario still tagged "@captain"
    And the inventory the provisional planks were read from is not empty

  Rule: a focused run selects one scenario by name, and cucumber-rs refuses a tag filter alongside a name filter, so the name is what excludes a skeleton or a condemnation from a focused run. The name reaches the runner as a regex, so a name carrying a regex metacharacter would match the wrong scenario or none. Uniqueness of a name within its feature file is the other property the focused command rests on; the feature lint discharges that one through its no-dupe-scenario-names rule, so it is not restated here.

  @conformance
  Scenario: no scenario name carries a regex metacharacter
    Given the scenarios in the specs
    When each scenario name is read as the focused command would pass it
    Then no scenario name carries a regex metacharacter

  Rule: a tier budget is a ceiling rather than advice, so it needs a check that reads it. The sweep commands already append their wall clock to the weather record, so no new instrumentation is owed. The check reads the most recent sweep of each tier rather than every entry retained, because the record is append-only and unbounded: one historical outlier would redden the check permanently while saying nothing about what the suite costs now, and the only remedy left would be deleting run data to clear a red. Yesterday's weather is the next run's starting prior, so the last observation is the one a ceiling judges. The floor guards the producer rather than the run history: a budget declared for a tier whose sweep command records nothing could never be exceeded, and a check that reads an empty record would report a clean bill for a suite it never measured. Verifying the producer structurally keeps a fresh clone honest, where no sweep has run yet and there is nothing to compare.

  @conformance
  Scenario: the most recent sweep of each tier is within its declared budget
    Given the tier budgets in "RIGGING.md" and the weather record
    When the most recent recorded sweep for each tier is read against that tier's budget
    Then no tier's most recent sweep exceeds its budget
    And every tier declaring a budget has a sweep command that records its wall clock

  Rule: four scantlings carry no JSON Schema dialect because they are proof contracts discharged by their own checkers in verification support. Their own shape is unchecked: the checkers read them into typed structs, so a required key that is misspelled fails loudly, but a key the struct defaults fails silently. A misspelled requiredReferences empties half the assistant boundary contract and the attestation stays green.

  @conformance
  Scenario: every proof contract satisfies the proof-contract meta-schema
    Given the four scantlings that declare no JSON Schema dialect
    When each is checked against the meta-schema in "scantlings/proof-contract.schema.json"
    Then all four validate
    And the meta-schema forbids a property it does not name

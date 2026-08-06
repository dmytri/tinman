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
    And the rule set carries at least the plank-form, plank-presence, perturbation-quiescence, process-wide-env-mutation, killed-measured-child and unshared-corpus-read rules

  Rule: a double was matched by the text of its declaration, which is not the same as its name. A struct merely holding a field of a double's type reddened, and the mark that justifies a real double could not be read at all, because an attribute sits between the doc comment and the item and a rule sees a comment's shape rather than its content. Both distinctions need a reader, so this check left the rule set for a step definition beside the plank joins.

  Rule: finding the doubles is not mechanizable here, and two attempts to pretend otherwise are why this says so. A name-keyed filter matched none of the forty-eight types this tree declares, because these doubles are named for what they stand in for rather than for being stand-ins. A shape-keyed filter fares no better: the standing example is a real HTTP server on a real port, which production reaches exactly as it reaches a provider, so it satisfies no production trait and shares no structural property with the thing it replaces. A double at a network boundary looks like ordinary code, which is the point of it.

  Rule: so the enumeration stays a human judgment made at harbour, and what a run checks is the half a run can decide. A mark either names one of the three conditions the Verification agreement permits or it does not, and that is readable. This catches the failure that actually happens, which is a double marked with a gesture rather than a justification, and the mark surviving long after the condition that earned it. What it does not catch is an unmarked double, and no check here will: an empty filter passing every assertion downstream of it is how the previous two versions of this scenario stayed green while inspecting nothing.

  @conformance
  Scenario: every exceptional-double mark names a condition the agreement permits
    Given the exceptional-double marks in the verification support sources
    When each mark is read for the condition it names
    Then every mark names one of the three conditions the Verification agreement permits
    And the marks read are not empty

  Rule: the tool that decides which scenarios a change selects is the one thing here whose mistakes are invisible. Over-selecting costs a sweep and a role notices the clock; under-selecting reports a set that was never run, and nothing contradicts it. It restated the tier list in its own source once, on the day it was written, and filed every scenario of a tier the rigging had just gained under the default tier, whose sweep excludes them by tag. Nothing caught that except a role sweeping the tier separately rather than believing the report. The rigging declares the tiers, so the tool is joined to that declaration rather than trusted to carry a copy.

  Rule: the fault this tool commits is silence, not error. It filtered the diff to Rust files and printed the same line for a change touching a scantling, a spec, the rigging or itself as it printed for a change touching nothing, so a role read that and ran nothing with no reason to doubt it. What pins it is not the mechanics of each join but the property the silence broke: a changed path is accounted for, either by the scenarios it selects or by being named as one no join reaches. Naming it is enough, because the override sweeps the tier an unresolved path serves; going quiet is what cannot be recovered from.

  @conformance
  Scenario: every changed path is selected for or named unresolved
    Given a diff touching a spec, a scantling and a path no join reaches
    When the selection tool reports the set that diff selects
    Then every changed path is selected for or named unresolved
    And the changed paths read are not empty

  @conformance
  Scenario: the selection tool recognises every tier the rigging declares
    Given the tier tags declared under "## Tiers" in "RIGGING.md"
    When the selection tool reports the tiers it recognises
    Then every declared tier is one it recognises
    And the declared tiers read are not empty

  Rule: the scanner reads functions and not items, so this gate covers duplicated functions only. What it cannot see is covered by a second census rather than counted here, at features/methodology-conformance.feature:every duplicated constant the census reports is named as coincidence. A count in this prose would decay every time the tree gained or lost a copy, and would decay silently, because nothing about a sentence changes when its number stops being true.

  Rule: the scanner fingerprints shape with identifiers normalised, so it cannot tell a copied body from a shape two unrelated functions share, and every exemption this gate once carried described the second. The size the scantling names is what separates them: across a substantial body, structural identity is copied logic. So the gate reads zero groups rather than a named set, and the floor moves to what the scanner analysed, because zero groups is the healthy resting state here and a scanner that read nothing reports it identically.

  @conformance
  Scenario: no duplicated body survives above the size the scantling names
    Given the implementation sources and the threshold in "scantlings/duplication-allowance.json"
    When the duplication scanner reads the sources at that threshold
    Then it reports no duplicate group
    And the code units it analysed are not empty

  Rule: the duplication scanner reads functions and not items, so a duplicated constant is invisible to it. Its unit census names closures, methods, functions and trait impl blocks and carries no unit kind for a constant at all, which means no threshold and no flag reaches one. The blindness was proven structural rather than a threshold effect on 2026-07-31 by a two-file probe: two byte-identical sources holding only constant declarations analyse as zero code units, so lowering the node and line minimums to 1 changes nothing. A green from the scanner therefore covers duplicated functions only, and a second census is what covers the rest.

  Rule: a duplicated constant is worth its own census because the constants that get copied are the ones that encode a decision. A deadline, a key sequence or a limit repeated in two files is two places to change and one place to forget, and the copy stays green while it drifts, because nothing joins the two declarations. This allowance is read in both directions, and the second direction is the one that rots: reading only that every reported constant is allowed lets an entry outlive the duplication it excused, so the copy is collapsed, the census stops reporting it, and a permission nobody needs sits in the file granting cover to whatever later matches it. Nothing announces that, because the gate it weakens stays green throughout. The function gate needs no such reading, because it names no entries at all.

  Rule: a constant is keyed by its name and its declaration together, and keying on the declaration alone was tried first. That key reads two constants agreeing in value as one duplicate, so unrelated deadlines that happen to sit at the same two seconds arrive as copies, and each would need a permanent allowance entry excusing duplication that never existed. A copy is one name declared twice. Two independent decisions that coincide are not a copy, and a census that cannot tell them apart fills its own allowance with noise until nobody reads it.

  @conformance
  Scenario: every duplicated constant the census reports is named as coincidence
    Given the implementation sources and the allowance in "scantlings/duplication-allowance.json"
    When the constant census reads the sources
    Then every duplicated constant it reports is one the allowance names
    And every duplicated constant the allowance names is one it reports
    And the constants read are not empty

  Rule: a wait on a spawned child that carries no deadline hangs the whole suite rather than failing one scenario, and it fails in the worst shape a run has: no output, no red and no weather line, because the sweep never finishes. The tier budget check cannot catch it either, since that check reads a record a hung sweep never wrote. The remedy is already in the verification support sources, which carry a helper that polls a predicate toward a deadline and reports whether the process exited in time.

  Rule: this is a reader's check rather than a rule entry, and it was tried as a rule entry first on 2026-07-31. A syntax query sees the shape of a call and not the type of its receiver, so the reader on this project's own terminal session is indistinguishable from a wait on a child process: both are a method call on a plain binding. The rule reddened four correct readers alongside the real faults, and narrowing it to the unambiguous wait forms would have missed the fault that motivated it. Where a check cannot identify its own subject it either matches nothing or matches the wrong thing, so the subject is given a findable form here instead: the census reads the process construction sites and asks what bounds each wait that follows.

  @conformance
  Scenario: every wait on a spawned child is bounded by a deadline
    Given the process waits in the verification support sources
    When each wait is read for the deadline that bounds it
    Then every wait on a spawned child reaches a deadline
    And the waits read are not empty

  Rule: escalation is owed when a termination deadline passes, and it is owed only then. Signalling a process that has already gone produces a diagnostic from the system's own kill, and that diagnostic reaches the runner's error stream because the shell running the signal inherits it. A sweep has printed about 34 such lines, which is harmless to results and is exactly the noise a real diagnostic hides behind. A stream carrying routine noise is one nobody reads.

  @conformance
  Scenario: tearing down a session whose process has exited leaves no failed signal
    Given a driver session whose process has already exited
    When verification support tears the session down
    Then the teardown reports the process had already gone
    And no diagnostic from the system's kill reaches the runner's error stream

  Rule: the durable prose cites scenarios, which is the right thing to cite, and nothing checks that the citations resolve. A scenario title is a durable artifact where a function name is disposable, so pointing at one points at the contract rather than at the machinery beneath it. But a title moves when a check's premise changes, and the sentence citing it then describes a scenario the tree does not carry, silently, because prose has no reader. This file was refitted three times in one day for that class of drift.

  Rule: so a citation takes a form a reader can find. A bare title in backticks is indistinguishable from any other backticked string, and a checker guessing which is which repeats the fault it was written to close: a filter that cannot identify its own subject either matches nothing or matches the wrong thing. The reference form the watchbill already uses names the file and the title together, so a citation announces itself and resolves exactly, and a document is free to keep quoting a title in prose beside it for a human to read.

  @conformance
  Scenario: every scenario a shipped document cites is one the specs carry
    Given the scenario references cited in the shipped Markdown documents
    When each is matched against the scenarios the specs declare
    Then every citation names a scenario the specs carry
    And the citations read are not empty

  Rule: the shipped Markdown documents are the tracked files the package manifest does not exclude, which is the set the citation check above reads. Defining the set by the manifest rather than by a list kept here means a document added to the tree is surveyed the moment it is tracked, and one the manifest excludes is never opened. Captain's private notes sit outside the set by that exclusion, and they are transient by design, so a membership claim in them decays without costing a reader anything.

  Rule: a membership claim and the sentence forbidding one carry the same words, so a check keyed on the word alone reddens the rule that defines the fault. Not one mention of the watchbill in the shipped documents is a claim: they are the rule, its reasoning, and a settled condemnation recorded so a later harbour stops re-deriving it. What separates a claim from a definition is that a claim names which scenario stands on the watch, where the definition speaks of a scenario generically.

  Rule: so the subject is given a findable form rather than a wider filter, which is the shape the child-wait census above already took. A claim is a membership construction on a line that also names a scenario or a spec, and both instances this closes took that form: one said a newly authored doubles check stood on the watchbill and named it, the other said the same of a manifest join. Each was true the day it was written and false the moment custody struck the file, and nothing announced the change, because a struck watchbill leaves the sentence exactly as it was.

  Rule: the count rule stated beside this one in the durable-prose rules gets no check, and admitting that is better than a filter that cannot name its subject. A decaying count is a live claim about what the tree carries now, and the shipped documents carried thirty-six bare integers on 2026-08-02 that were status codes, byte counts, sweep timings, package deltas and versions. Telling a live count from a dated measurement means reading which date governs a sentence, which is discourse rather than a line, and the marginal case is already in the tree: the download figures under the proxy decision carry their date in a neighbouring sentence rather than their own. That enumeration stays a judgment made at harbour.

  @conformance
  Scenario: no shipped document claims a scenario it names is on a watchbill
    Given the shipped Markdown documents
    When each line naming a scenario or a spec is read for a watchbill membership claim
    Then no line claims the scenario it names is on a watchbill
    And every shipped Markdown document was read

  Rule: nothing reads the outbound section. Every other rigging value that matters carries a check: the tiers have the budget scenario, the plank commands have their join, the scantlings have their reachability. The one section only a release executes had none, and three of its four values were wrong through two releases, one of them shipping schema URIs that answered 404 from the day they were published. A release is too rare to be the thing that finds a broken release command.

  @conformance
  Scenario: every outbound target carries a ship and a verify that report their own status
    Given the outbound targets in "RIGGING.md"
    When each target's ship and verify lines are read
    Then every target carries both a ship line and a verify line
    And every one of those lines reports its own exit status
    And the targets read are not empty

  Rule: a scantling and its published URI are read over the network by consumers who never run this suite, so both are checked here. A scantling that declares a dialect must satisfy it: a mistyped keyword yields a schema that validates everything and an attestation that asserts nothing. Each count is named so an empty read fails rather than passes; the proof contracts carry no dialect because they are discharged by their own checkers instead. The counts live in the scenario steps, where a run checks them; restating one here would only decay, since nothing about a sentence changes when a scantling is added.

  @conformance
  Scenario: every scantling declaring a dialect satisfies it
    Given the scantlings that declare a JSON Schema dialect
    When each is checked against the JSON Schema 2020-12 meta-schema
    Then all ten validate

  @conformance
  Scenario: every published schema URI names the packaged version
    Given the package version in "Cargo.toml"
    When the schema URIs in the scantlings and the example plans are read
    Then all twenty-four name that version

  Rule: the project ships two artifacts from one tree, the crate and the npm package, and each carries its own version field. A bump applied to one is invisible to the other, and the schema-URI check above reads the crate manifest alone, so before the scenario below a forgotten npm manifest passed every gate and reached the registry naming a version that described different contents. The registry is where that would be discovered, which is after it is published and cannot be taken back.

  @conformance
  Scenario: both packaged manifests name one version
    Given the package version in "Cargo.toml"
    And the package version in "npm/package.json"
    When the two are compared
    Then they are the same version

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

  Rule: a background declared before any rule is feature-scoped and reaches every scenario, including scenarios inside later rule blocks. A background declared below a rule belongs to that rule, so the next rule ends its scope and every scenario after it runs with its given state unprovisioned. Those scenarios still run and still report, which is what makes it expensive: they fail on a precondition nobody removed on purpose, or they pass while asserting against state the background was supposed to have built.

  Rule: so the position that is safe is the earliest one, and it is safe permanently rather than until the next edit. A background sitting below the rules works only while no rule follows it, which makes a correct file one edit away from a broken one, and the edit that breaks it is the ordinary act of adding durable context. Placing the background above every rule removes the failure mode instead of avoiding it.

  Rule: this check exists because the prose form of it failed three times, and the first version of the check was itself wrong. The trap reddened seven scenarios once, was written up as a note, and recurred twice inside one session on 2026-07-31, the second time minutes after that note was rewritten to forbid it more forcefully. The check first written to close it asserted that no rule may follow a background at all, which is not true of this runner and reddened six correct scenarios across two features. The rule that is true is narrower and is the one stated here.

  @conformance
  Scenario: every background is declared before any rule
    Given the feature files in the specs
    When each feature carrying a background is read for a rule declared above it
    Then every background is declared before any rule
    And the features read are not empty

  Rule: a worker count is a claim about how a suite runs, and this one was never true. The runner polls every scenario future on a single executor thread, so a step body that blocks on a real process or sleeps holds that thread and the scenarios beside it wait. Measured on 2026-08-02, one feature ran 18795ms at one worker, 18778ms at four and 18790ms at eight, and a scenario costing 19ms alone recorded 5146ms inside a wave. The recorded cost was overlap rather than work, which is why the tier looked expensive and no fixture was to blame.

  Rule: the check is the comparison rather than the count, because a count is what the rigging already claimed. A tier that runs no faster with four workers than with one has a concurrency value that documents an intention, and every conclusion drawn from its wall clock inherits that. Reading a budget against a serialized tier measures the executor rather than the suite.

  @sandbox
  Scenario: a tier runs materially faster with four workers than with one
    Given a fixture feature whose four scenarios each block on a real sandboxed process
    When it is run at one worker and again at four
    Then the four-worker run takes less than three quarters of the one-worker wall clock

  Rule: the budget check reads a whole-tier wall clock, so it can say a tier outran its ceiling and never say which scenarios spent it. Amortizing a fixture is only worth spending on the scenarios that carry the cost, and picking them without measurement is the guess this project keeps paying for. So the ceiling and the attribution are two checks: one says the suite has outgrown its budget, the other says where the time went.

  Rule: this was recorded as needing a bespoke writer, and that was wrong twice over. The runner's own shipped writers do carry per-scenario timing, so no bespoke writer was ever owed. Each of them also pulls new crates, so neither is the route taken here, and the measurement costs no dependency at all. The correction is named because the false claim is what deferred the work, and a cost believed unmeasurable is a cost nobody measures.

  Rule: the record holds every run, and only some of them are sweeps. The concurrency check above runs generated fixtures through the same recorder, so a reader taking whichever complete run came last can attest four fixture scenarios while reporting on a tier. It passes either way, which is why the substitution never announces itself and why the sweep is identified rather than assumed.

  Rule: a run whose filter matched no feature is not a sweep either, and the runner cannot be leaned on to say so: it reports no features, no scenarios, and exits clean, which is the same silent pass a glob reading no file gives. A completion line written for such a run makes the next reader attest an empty set, and the red then lands a run late and names the empty sweep rather than the invocation that wrote it.

  @conformance
  Scenario: the wake records how long each scenario took
    Given the most recent tier sweep the wake recorded
    When the durations it recorded are read
    Then every scenario it ran carries its own duration
    And the sweep read covered a whole tier rather than a generated fixture
    And the durations read are not empty

  @conformance
  Scenario: a run that matched no scenario is not offered as a sweep
    Given a run whose filter matched no feature
    When the durations the wake recorded are read
    Then that run is not offered as a sweep
    And the runs read are not empty

  Rule: a tier budget is a ceiling rather than advice, so it needs a check that reads it. The sweep commands already append their wall clock to the weather record, so no new instrumentation is owed. The check reads the most recent sweep of each tier rather than every entry retained, because the record is append-only and unbounded: one historical outlier would redden the check permanently while saying nothing about what the suite costs now, and the only remedy left would be deleting run data to clear a red. Yesterday's weather is the next run's starting prior, so the last observation is the one a ceiling judges. The floor guards the producer rather than the run history: a budget declared for a tier whose sweep command records nothing could never be exceeded, and a check that reads an empty record would report a clean bill for a suite it never measured. Verifying the producer structurally keeps a fresh clone honest, where no sweep has run yet and there is nothing to compare.

  @conformance
  Scenario: the most recent sweep of each tier is within its declared budget
    Given the tier budgets in "RIGGING.md" and the weather record
    When the most recent recorded sweep for each tier is read against that tier's budget
    Then no tier's most recent sweep exceeds its budget
    And every tier declaring a budget has a sweep command that records its wall clock

  Rule: the assistant contract restates the model's style object rather than referencing it, so two schemas hold one shape. That is tolerable only while something keeps them in step: when the model gained six standard attributes the assistant contract still required four, and a model emitting the old four would have failed the model's own schema while passing the interface's. Neither file announces the drift, and both stay green apart. A cross-file reference would remove the duplicate outright, and no scantling here uses one yet, so the join stands until that is proven rather than assumed.

  @conformance
  Scenario: the assistant contract requires the style the model defines
    Given the style properties required by "scantlings/tom.schema.json"
    And the style properties required by "scantlings/assistant-ui.schema.json"
    When the two sets are compared
    Then they are the same set
    And the properties read are not empty

  Rule: a sandboxed scenario creates a real process and a real staging directory, and a run that is killed or crashes cannot be trusted to have torn either down. Reclaim at suite start is the safety net for exactly that, and a net nobody checks reports the same clean bill whether it is holding or not. What it costs when it stops holding is not tidiness: an orphaned sandbox keeps a bind mount alive against a directory the operator may be editing, and staging directories accumulate silently until the disk answers for them.

  Rule: the floor here is the prefix set rather than the count, and the earlier floor is why. Reporting that the inventory searched something distinguishes an unread inventory from an empty one, and it cannot distinguish an inventory that searched the wrong place: this check stayed green across four days while ninety-nine directories accumulated, sixty-one of them under a copy-mount prefix the search never named and thirty-three under a terminfo prefix, beside an orphaned sandbox process three days old holding a bind mount. The implementation is what decides which prefixes exist, so the implementation is what the search is measured against; a prefix added later that nobody adds here reddens rather than accumulating in silence.

  @conformance
  Scenario: no sandbox resource outlives the run that created it
    Given the sandbox processes and staging directories present after the suite reclaims
    When each is matched against the runs that are still live
    Then no sandbox process outlives the run that created it
    And no staging directory outlives the run that created it
    And the inventory searched every temporary-directory prefix the implementation creates

  Rule: the proof contracts carry no JSON Schema dialect because they are discharged by their own checkers in verification support. Their own shape is unchecked: the checkers read them into typed structs, so a required key that is misspelled fails loudly, but a key the struct defaults fails silently. A misspelled requiredReferences empties half the assistant boundary contract and the attestation stays green.

  @conformance
  Scenario: every proof contract satisfies the proof-contract meta-schema
    Given the thirteen scantlings that declare no JSON Schema dialect
    When each is checked against the meta-schema in "scantlings/proof-contract.schema.json"
    Then all thirteen proof contracts validate
    And the meta-schema forbids a property it does not name

  Rule: a scantling creates no work until something references it, so an unreferenced one is a contract nobody discharges while every attestation stays green. Four of the boundary contracts are reached through a path literal in a step definition rather than through a path named in a scenario, which is the sound route and the reason this join reads both. The floor guards the reader: a listing that finds no scantlings and a directory that holds none both report zero.

  @conformance
  Scenario: every scantling is reached by a scenario or a step definition
    Given the scantling paths under the scantlings directory
    When each is matched against the specs and the step definitions that read it
    Then every scantling path is reached by at least one of them
    And the scantling paths read are not empty

  Rule: nothing in this tree reddens when an advisory lands. Both advisories found so far were found because a role went looking, one by a harbour audit and one by a role reading a 200 as publication for a package abandoned since 2024. A fact discovered only when somebody remembers to look is a fact the suite is not carrying. The shipped graph is the one that matters, because what ships is a Rust binary and the npm graph is dev rigging no released artifact carries. An advisory is queried by name and version together, since a name-only lookup returns a hit that reads as live against a pinned version that is unaffected, and a false alarm reopens a settled pin.

  @conformance @advisory
  Scenario: no advisory reaches a crate in the shipped graph
    Given the crates the release graph carries
    When each is queried against the advisory database by name and version
    Then every crate in that graph was queried
    And no advisory reaches any of them

  Rule: durable prose that cites machinery rots every time the machinery improves, and the better the check gets the faster it rots. Only part of that rule has a findable subject. A sentence hardcoding a live count was declined on a measurement, because most bare integers in this corpus are status codes, byte counts and versions, and separating a live count from a dated one means reading which date governs a sentence. A sentence recording that a past decision was wrong is discourse and no query reaches it. What a query does reach is a citation of the implementation tree itself, which is the fault this rule exists for: prose naming a source path outlives the seam it names.

  @conformance
  Scenario: no rule body cites a path under the implementation directory
    Given the rule bodies the specs carry
    When each is read for a path under the implementation directory
    Then every rule body was read
    And no rule body cites such a path

  Rule: a tag on a rule line is inherited by every scenario inside that rule, which is useful for a tier tag and a trap for a lifecycle tag. A tier tag has no promotion path, so inheriting it is how a whole rule declares its cost once, and this corpus uses it deliberately. A lifecycle tag is workflow state that ends by being deleted, and deleting the one on the scenario leaves the one on the rule still excluding it from every derived command. Nothing announces that: the scenario count does not move, no run reddens, and the promotion silently does nothing. Captain wrote this fault twice in one session, the second time into the skeleton arguing against prose faults, so the placement wants a check rather than more care.

  @conformance
  Scenario: no lifecycle tag sits on a rule line
    Given the lifecycle tag lines the specs carry
    When each is read for what it precedes
    Then every lifecycle tag line was read
    And none of them precedes a rule

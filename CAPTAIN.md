> STOP. Captain's notes: non-binding. Captain writes, Captain trims. Anyone else: close this file now.

# Captain Notes

Binding behaviour lives in `.feature` specs and referenced scantlings. History lives in git. These notes carry only what the next cycle needs.

## First action

**The deck is not yours yet. QM has it.** Voyage 4's 16 targets all came back green, then Boatswain's recheck fouled the deck on a red that predates the voyage. Voyage 4a is authored to fix it: `watchbill.json` carries the one red plus the three owed tier sweeps. The next role is QM in cleared context.

Captain resumes when QM reports the watchbill spent and Boatswain reports custody. Captain edits ride uncommitted meanwhile: the struck `budget` line in `RIGGING.md`, this file, and voyage 4's durable artifacts. That is work in flight and Boatswain commits it with the voyage's custody. Do not treat it as a dirty deck to clean.

Run the Opening retrieval anyway and believe it over this paragraph.

## Deck state, 2026-07-26, voyage 4 verified and fouled at custody

Voyage 4 ran: QM took all 16 targets green at one deck state, wrote 16 run-record lines, dispatched five Crew mates and proved three new checks red before green. Boatswain then refused custody. **Nothing is committed. Base commit is still `4f1b85b`.**

**The foul: `features/methodology-conformance.feature:every published schema URI names the packaged version` is red.** `tests/cucumber/support.rs:1545` panics `scantling {display} publishes no $id` because `published_schema_uris()` treats every `.json` under `scantlings/` as published. `scantlings/proof-contract.schema.json` says in its own `description` that it "is not published under a versioned URI". The durable artifact settles the intent; the support code contradicts it. Fix is QM's, in place: skip a scantling with no `$id`, exactly as `dialect_scantlings()` skips one with no `$schema`. The `all thirteen name that version` count then stands.

**The red predates voyage 4.** `git log -- scantlings/proof-contract.schema.json` reports one commit, `4f1b85b`, and that commit's own message records that no tier sweep was run. This is the cost that commit named coming due. Do not hunt for a voyage-4 regression.

**`broad-inference` is owed.** Boatswain ran `broad` (131/132, the one red above) and `broad-sandbox` (34/34 green) fresh, and left the paid tier to the fixing role because the deck was already foul. Voyage 4a's watchbill orders all three sweeps, cheapest first, because one cucumber binary serves all three tiers and `TinmanWorld` changed.

**The `budget` strike in `RIGGING.md` is Captain's own edit, not orphaned dirt.** Boatswain correctly could not attribute it from the diff and left it unstaged twice now. It is the user's decision recorded under "Settled: no budgets" below, written under the write-scope exception that gives Captain a tier's `budget` value. Tell Boatswain it is Captain work in flight when dispatching custody, or it will stall on it again.

Tags `v0.1.1` and `v0.1.2` are on origin; crates.io carries 0.1.0, 0.1.1 and 0.1.2, none yanked. 0.1.2 is correct **as published** and needs no yank. The working tree has diverged from it, so the next release is a bump rather than a republish, and the `fixture` strike makes that bump breaking.

## Scheduled work

Every item names its pivot and its role. No "someday".

**Voyage 4a, live now, QM through the watchbill.** One red plus three tier sweeps. See the foul above.

**Voyage 5, Captain authors next, and the first item is the biggest.** Four items, all earned by voyage 4's own reports.

1. **The `cwd` divergence is still open. Voyage 4 did not close it, and the notes that claimed the plan were wrong about what Crew would do.** Confirmed by command after custody was refused: `src/pty.rs:111-115` shows `capture_interactive_in` **still taking `cwd: Option<&Path>`** as a third parameter, `src/bwrap.rs:99-100` still deriving `--chdir` from its own `cwd` argument, and `src/process.rs:14` carrying `PreparedProcess.cwd` that is set `None` at all three construction sites, `inspect.rs:24`, `record.rs:200` and `bwrap.rs:190`, and **read nowhere in `src/`**. `src/flow.rs:68` passes `run.cwd.as_deref()` down the sidecar, so the flow scenario is green on the sidecar path and the `@contract` scenario is green because the field serializes. Both honest, neither touching the divergence. The pin this needs is structural, not behavioural: nothing a user consumes is broken, so the constraint is that the working directory reaches the process **through** the prepared process, per `AGENTS.md`. Prefer a proof contract on the module, the `forbiddenReferences` branch of `scantlings/proof-contract.schema.json`, over a perturbation; read the existing contracts and their checker before authoring, because the branch constrains *named symbol references* and a parameter name may not be within its reach. Reach for a perturbation only if it genuinely cannot be pinned.
2. **`the driver exits when its stdin closes` asserts nothing in its second step.** `And the driver leaves no session sandbox directory standing` cannot fail: the only session-directory creation is `src/driver.rs:179-180` inside `launch`, and that scenario's Given launches nothing. The step definition is honest and scoped to the driver's own pid; the precondition is the defect. Strengthening the Given to run a session gives it teeth and moves the scenario to `@sandbox`, since its assertion becomes isolation itself.
3. **`scantlings/driver-protocol.schema.json` declares no `required` list on `params`.** Its own description reads "Every method except launch addresses an existing session", and both new invalid-params scenarios send captures carrying no `session` and validate anyway. The missing-parameter scenario omits two parameters while naming one. Prose that binds nothing is the same fault as a property outside `required`. Owe a per-method `required` list.
4. **No derived check reddens provisional-plank drift.** QM found all five spent `@planks-provisional` annotations by hand. The rule set carries plank-form, perturbation-quiescence and forbidden-doubles only, and both plank joins in `features/methodology-conformance.feature` cover `@planks` alone. The Planking agreement says every state it names carries an act; this one has no check. Derivable now, not a harbour deferral.

**Next release, whoever ships.** `sandbox-spec.schema.json` lost the `fixture` values from `home` and `env.*.from` at the 2026-07-25 harbour. That is a breaking change to a published schema, so the release carrying it owes a version bump. The bump still has no owner in doctrine; see the gaps below.

**Next harbour, Shipwright.**
1. **`src/driver.rs` reads 0.00% in every tier** while its 24 binding scenarios pass. `tests/cucumber/support.rs:952` SIGKILLs the child, and a killed instrumented process never flushes counters. The whole out-of-process surface is unattributed, so no coverage claim about the driver means anything until this is closed.
2. **`forbidden-doubles` keys on the type names `Mock|Fake|Stub|Dummy`.** `LocalProvider` is a real double, correctly marked, that the rule would not have caught unmarked. Any double named otherwise is invisible.
3. **Derive a schema-property-against-production-field check.** Both `cwd` divergences survived because a scantling only validates the shape production emits, so a declared property production never emits is untested in both directions. Two schemas drifted from two structs and every check stayed green. See the corollary under the false-green section.
4. **Decide `ratatui-testlib`'s harness layer.** Leaning no on architecture: launch must happen inside bwrap and the PTY runner accepts only a prepared process, while `TuiTestHarness` owns spawning and ships no sandbox. Six weeks at v0.1.0 with one maintainer likely fails the well-maintained bar for a foundational layer.

**Held, not scheduled.** `jsonschema` 0.48.5 and `generic-array` 0.14.7 sit behind latest with 0 semver updates available. The `locked` policy holds them until a spec or a Captain decision moves them. No spec needs either. Revisit at next harbour.

**Settled: no budgets.** `budget: 120s` is struck from `## Tiers` by user decision. It was a ceiling no command produced and no check read, and `@inference` alone had exceeded it. Better no budget than an unenforced one. Observed floors if one is ever wanted: `@logic` ~24s, `@sandbox` ~26-34s, `@inference` ~63s with one 348s outlier during the flake. Reintroducing a budget needs a producer first, since the `coverage` commands deliberately skip the weather append and harbour's own regression therefore records nothing to check against.

## The false-green pattern. The most expensive thing this project has learned.

Twelve instances now, every one green while asserting nothing. The six older ones: a perturbation discharged by deleting its line; a `menu` role the deterministic pass already produced; a launch answering `ok:true` before the program started; a recorded plan carrying no expectation; two activation scenarios asserting text the fixture drew anyway; and a scantling that is valid JSON and an invalid schema.

Two more this harbour, both caught only by planting:

- **The gplint glob.** `features/**/*.feature` matches zero files in gplint's glob, because `**` must consume at least one directory segment and `features/` is flat. It exits 0 having linted nothing. Captain and Shipwright hit this independently, an hour apart, both reaching for the idiomatic form. The working values are `features/**` and `features/*.feature`. `RIGGING.md` carries the former and `AGENTS.md` records the trap.
- **A plank string shared across six seams.** `@planks("a process is prepared and launched")` is carried by `process.rs`, `sandbox.rs` three times, `bwrap.rs` and `pty.rs`. The string join is green no matter which of the six is dead, so a shared plank string makes the join structurally unable to see a dead seam. The trap is real; the seam it was found on turned out not to be dead. See the negative-control rule below.

Two more on 2026-07-26, both `cwd`, and the second is the worst class yet because no scenario covered it at all:

- **An optional schema property is an attestation that cannot fail.** `prepared-process.schema.json` declared `cwd` outside `required`, so a `PreparedProcess` that has never carried the field serializes without it and validates. The `@contract` scenario passed for two full cycles while the struct and the schema disagreed. Any optional property in a scantling is a promise nothing checks: read the `required` list, not the property list, when asking whether an attestation binds.
- **A schema can promise a capability production silently discards.** `harness-plan.schema.json:51` declares `cwd` on a `run:` step, and `additionalProperties: false` there means the key is explicitly *permitted*, not tolerated. `RunForm::Full` in `src/plan.rs:57-63` has `command`, `status` and `stdin` and no `cwd`, and the enum is `untagged` with no `deny_unknown_fields`, so serde drops the key without a word. A plan writing `cwd: subdir` validates against the published schema and then runs in the wrong directory. No error, no warning. This is quieter and therefore worse than the `home: fixture` case, which at least failed loudly at deserialize. Grep found it in one command: no reader of `cwd` existed anywhere in `src/` outside `pty.rs`.

Two more from voyage 4's own reports, and the first is the one to study, because a lever designed against this exact pattern failed:

- **A `required` property proves a field exists. Nothing proves it is read.** The note above congratulated itself on moving `cwd` into `required` so that Crew could not thread another sidecar and skip `PreparedProcess`. Crew added the field, left it `None` at every construction site, read it nowhere, and threaded the sidecar anyway. Both scenarios went green honestly. Tightening a scantling raises the floor by exactly one inch: the field's presence and type. The capability's *wiring* is a different claim and needs a different check.
- **A precondition that seeds nothing makes its assertion vacuous.** `the driver exits when its stdin closes` asserts no session sandbox directory is left standing, in a scenario that launches no session. Four green steps, one of them asserting against a state that could not exist. This is the Given-side twin of the older cases, which were all assertion-side: the step definition is correct and the setup starves it.

**The test: when a scenario goes green early and cheaply, ask what it would take to make it red. If nothing would, the scenario is the defect.**

**Corollary, now twice earned and sharper than before: a scantling only ever tests the shape production emits.** A property production never emits is untested. A property production emits but never *reads* is also untested, and that one survives a `required` list. Both `cwd` instances were found by hand, prompted only by an old note. The join that would catch either is schema property against production field, plus a reachability question the schema cannot ask: does anything consume it? Nothing we own runs that. Worth a derived check, and item 1 of voyage 5 is the manual instance of it.

Three corollaries, the third earned this harbour:

- **Where durable context changes and no scenario reddens, write the scenario that reddens.** Reach for a perturbation only where behaviour genuinely cannot be pinned.
- **A ruling that widens what one layer owes narrows what every other layer may claim.** Re-read every scenario that counts on the definition in the same pass. Adding the proof-contract meta-schema moved the dialect count from eight to nine, and the counting scenario had to move with it.
- **A green from a subagent is a claim until a command answers it.** Shipwright pass two reported four scenarios exercising `pty::launch`; a coverage run showed those scenarios executing `pty::capture` at count 4 and every `launch` closure at count 0. Both passes were honest; the plank string was shared and the join could not tell them apart.

**Zero coverage on a negative-control branch is the control working, never evidence of dead code.** Captain read `pty::launch` at count 0 and ruled it dead; Shipwright refused the removal and was right. `tests/cucumber.rs:311-374` points the backend at a deliberately absent bwrap, builds `/bin/sh -c "touch <sentinel>"`, and asserts at line 367 that the sentinel does **not** exist. `tinman::pty::launch` at line 349 is the only call that could ever create it, so deleting the seam turns that assertion into a tautology that passes whether or not the sandbox refused anything. A branch that never runs because a guard held is the opposite of a dead branch. Before removing any zero-count code, find the assertion that would go quiet with it.

Two further facts that settle the seam. `pub mod pty` in `src/lib.rs` plus `pub fn launch` makes `tinman::pty::launch` public API of a published crate, reachable by a library consumer the binary never exercises. And `pty::capture` has no production caller either: production imports only `capture_interactive`, `capture_interactive_in` and `capture_interactive_at`. Coverage count separated the two; reachability does not. **Both seams are kept.** A future harbour re-deriving "dead code in `src/pty.rs`" is repeating a settled decision.

## Rules of the language and the tooling

- **A `Background:` belongs to the `Rule:` it sits under.** Adding a `Rule:` above a `Background:` orphans it for every scenario after. This reddened seven scenarios once. No feature file does it now; check before adding one.
- **gplint's `indentation` rule is off deliberately.** It expects scenarios to nest one level under a `Rule:`, Scenario at 4 and steps at 6. The house style is flat 2/4 throughout. Re-enabling it means reindenting the corpus for no semantic gain.
- **gplint's dupe-name option is `in-feature`, not `in-file`.** One Feature per file makes them the same thing here. gplint owns name uniqueness now; the `@conformance` scenario for it was dropped rather than hold one rule in two places.
- **Read the registry before naming a version.** `cargo search tinman` answers it in one command and is already the `## Outbound` verify line. Recommending a `v0.1.1` tag without it once cost a force-moved tag on a public remote.
- cucumber-rs makes `--tags` and `--name` mutually exclusive. Tag exclusion rides `CUCUMBER_FILTER_TAGS`; `--name` selects the scenario. Encoded in `RIGGING.md` `focused`, which therefore cannot satisfy the Rigging read contract's tag-exclusion clause; the anchored name does that job instead.
- Runner is `tests/cucumber.rs`, `harness = false`, `fail_on_skipped()` so undefined steps redden.
- No clean cucumber-rs dry-run, so `discover: none`. To prove specs parse, run the default tier and read the feature count.
- Read the `result` field before trusting a weather line: `101` is a cargo test failure, so a fast line with `result: 101` is an early abort wearing a duration.
- Env confirmed: rustc 1.97 (edition 2024), bwrap 0.11.2, user namespaces enabled, node 22.23.1. `.env` holds a live `TINMAN_API_KEY`, git-ignored, so `@inference` is runnable and costs money per run.

## Two doctrine gaps worth raising upstream

1. **A release version bump has no owner.** The write-scope list gives Shipwright the manifest for dependency work only, and Shipwright is a harbour role while a release is mid-outbound. Captain bumped `Cargo.toml` to 0.1.2 under Captain's authority at sea, recorded as a departure. Boatswain named the same gap independently. It recurs at the next release, which now owes a bump for the `fixture` strike.
2. **An adoption proof leaves no durable trace.** A green looks identical proven or not. Five proofs now: the proof-contract meta-schema and the gplint command at the 2026-07-25 harbour, and three checks QM planted red on voyage 4. Every one of them was proven in a session that has since been cleared, so the tree carries no record that any of them can fail. A proof only its author can see is one clear from being unproven again. The run record has a slot-shaped hole here: it records that a target passed, never that it was ever observed to fail.

## Design decisions that bind (user-confirmed)

- **Tinman is a driver, not only a CLI.** Tests live in pytest, jest and bun test and drive Tinman as Playwright's clients drive the Node driver. `tinman driver` speaks newline-delimited JSON-RPC on stdin and stdout. The YAML plan stays canonical for recorded flows. The protocol is RPC, not a second test format.
- **TOM is the DOM equivalent and inference is codegen.** The deterministic builder is the spine and produces every addressable role from terminal idioms. The LLM engine is a second producer of the same shape, capture time only, proposing **names, never roles**. A hand-authored plan needs no model, which is why replay needs none.
- **Resolution and confirmation are two operations.** Resolution answers what a locator matches and reports ambiguity as ambiguity. Confirmation runs at capture time only. Collapsing them makes an ambiguous locator look bindable to the test that must later resolve it alone.
- **Terminal size is a property of the run, never of the plan.**
- **Plan YAML grows with the test.** Shorthand removes typing, never adds capability, never weakens a default. An omitted `sandbox:` block means secure defaults, not no sandbox.
- **`home: fixture` and `env.from: fixture` are struck.** They were declared in the published schema and absent from production, so a plan writing them validated and then failed to deserialize. `mounts` is the live provision for reaching a tree outside the bound set. Breaking changes are acceptable at 0.1.x.
- **The `@inference` tier asserts our seam, never the model's compliance.** `the provider's reply contains "READY"` asked a model to obey and flaked; it now asserts a reply was parsed and carries non-empty content. `a non-empty expansion is produced` was already correct and was left alone. Shaping the request is not validating the response.
- **Help text is an asset, not a scenario.** Copy lives in `assets/help/`. No acronym validation: the generator gets the bundled skill's name and description and nothing else.
- **Inference: any OpenAI-compatible provider, OpenRouter the default.** `TINMAN_API_KEY`, `TINMAN_BASE_URL`, `TINMAN_MODEL`. Environment wins over `.env`. The credential is vendor-neutral by name so the default is never lock-in.
- **Tier placement.** `@sandbox` marks scenarios whose assertion is isolation itself; ordinary PTY launches stay default tier. `@inference` is real paid calls, never on the inner loop. Retag a scenario rather than weaken a tier policy.
- **Emulator is `alacritty_terminal` 0.26.0.** `vt100` was unmaintained and failed cell addressing and reverse video. `wezterm-term` is better and disqualified: not on crates.io, and we publish. `alacritty_terminal` is 0.x and breaks across releases; budget for that.

## Standing preferences the user has stated

Read these before authoring any spec.

- **Prefer a concise attestation on a scantling over verbose behavioural scenarios**, and audit freshly authored specs against that, not only inherited ones. Applied again this harbour: one folded rule-set attestation replaced three per-rule methodology scenarios that restated one exhaustive scan three times.
- **A scantling based on a well-known standard beats a bespoke one**, and **prefer specifications over implementations**. Applied: JSON-RPC 2.0, WAI-ARIA, JSON Schema 2020-12. AccessKit's node shape was declined on this ground.
- **Prefer an independent tool over a bespoke checker**, and never let two checkers hold one rule. gplint's `no-dupe-scenario-names` took the name-uniqueness rule and the bespoke scenario was dropped.
- **`Rule:` prose carries durable context only.** Requirements belong in scenarios.
- **Tinman must stay useful for plain command-line testing**, not only full-screen work. A coding agent is both, and one suite should need one tool.
- **No backlog debt.** Do it, decline it, or name its pivot.
- Breaking changes are acceptable at 0.1.x, so a correct reshape does not wait for a major version.
- **Specify the floor as well as the ceiling.** A scenario asserting a measurement against a threshold also carries a null control. Executable specs make behaviour legible and calibration opaque. The worked case is `ratatui-testlib`: `assert_render_budget(60.0)` asserted input-to-render latency under 16.67ms while the harness held a measurement floor near 150ms, about nine times the threshold. Never instrument latency across a synchronization primitive we control. The rule generalizes past latency: `all nine` and `all thirteen` are floors, so a glob reading nothing fails rather than passes.
- **Auto-memory is off**, `autoMemoryEnabled: false` in both settings files. A memory store re-injecting Captain decisions into role sessions makes the Context bulkhead advisory. Anything worth keeping goes to a `.feature`, a scantling, `AGENTS.md`, `RIGGING.md`, or here.

## Prior art and the accessibility angle

`ratatui-testlib`, crates.io, MIT, v0.1.0, single maintainer, framework-neutral despite the name. It occupies Tinman's transport layer via the same `portable-pty` and carries no semantic model, no sandboxing, no replay, no inference. Snapshot testing was considered and rejected: `insta` asserts "same as last time", which is drift detection rather than specification, and a snapshot passes whatever it recorded including a regression recorded before anyone looked.

**Accessibility is the TOM's second consumer.** Not scope and not a promise; a standing reason to hold the model honest. Terminal accessibility is an empty category: VTE exposes a flat `AtkText`, Windows Terminal a UIA `TextPattern`, so an ncurses menu reaches a screen reader indistinguishable from a paragraph. A black-box model deriving ARIA roles and accessible names from an attributed cell grid is the missing piece, and the TOM already emits that shape. Two free consequences: a wrong role passes a test but a listener notices at once, so holding the model to what a human would need spoken aloud improves it for testing; and never route the TOM through a lossy intermediate, because attribute-derived structure needs a genuine attributed grid.

Keep the framework-neutral naming: `assets/skill/SKILL.md` says "CLIs and full-screen TUIs" and names no framework.

## Reference docs, not sanctioned artifacts

`idea.md`, `isolation.md` and `help.md` are intent sources in the crate's `exclude` list. Do not treat them as requirements. Note that `isolation.md:108` still shows `from: fixture`, which production never had and the schema no longer declares.

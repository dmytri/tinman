> STOP. Captain's notes: non-binding. Captain writes, Captain trims. Anyone else: close this file now.

# Captain Notes

Binding behaviour lives in `.feature` specs and referenced scantlings. History lives in git. These notes carry only what the next cycle needs.

## Deck state, 2026-07-26, after harbour

Harbour ran three Shipwright passes: inventory, condemnation plus gplint install, and an attempted seam removal that was correctly refused at its guard. Voyage 4 is authored; `watchbill.json` carries 5 watches and 14 targets, ordered cheapest tier first. Confirm the deck with commands, not with this paragraph.

**The branch is ahead of `origin/main` and unpushed.** Confirm the count with `git rev-list --left-right --count origin/main...HEAD`. Tags `v0.1.1` and `v0.1.2` are on origin; crates.io carries 0.1.0, 0.1.1 and 0.1.2, none yanked.

**A red `@sandbox` tier before QM is the expected state, not a regression.** The promoted scenarios have no step definitions yet and `fail_on_skipped()` reddens them. Observed at authoring: 34 scenarios, 30 passed, 4 failed, while watch4 names only 3 driver-protocol targets. Identify the fourth rather than assuming it is one of them.

Harbour's full regression was green before the promotions: `@logic` 125, `@sandbox` 31, `@inference` 3. Merged three-tier census 77.58% lines. 0.1.2 remains the correct published release; nothing under `src/` changed before this voyage.

## Scheduled work

Every item names its pivot and its role. No "someday".

**Next voyage, QM, through the watchbill.** 14 targets. Watch2 includes `every step definition binds at least one scenario`, which reddens on the five orphaned definitions in `tests/cucumber.rs` and removes them as its fix.

**Next voyage, Captain.** `cwd` is documented in `prepared-process.schema.json` and absent from `PreparedProcess`. The conformance scenario is green because the field is optional, so this is latent divergence. `AGENTS.md` says the PTY runner accepts only a prepared process, which argues the field lands rather than the scantling drops it. Carried from the last cycle, still unwritten.

**Next release, whoever ships.** `sandbox-spec.schema.json` lost the `fixture` values from `home` and `env.*.from` this harbour. That is a breaking change to a published schema, so the release carrying it owes a version bump. The bump still has no owner in doctrine; see the gaps below.

**Next harbour, Shipwright.**
1. **`src/driver.rs` reads 0.00% in every tier** while its 24 binding scenarios pass. `tests/cucumber/support.rs:952` SIGKILLs the child, and a killed instrumented process never flushes counters. The whole out-of-process surface is unattributed, so no coverage claim about the driver means anything until this is closed.
2. **`forbidden-doubles` keys on the type names `Mock|Fake|Stub|Dummy`.** `LocalProvider` is a real double, correctly marked, that the rule would not have caught unmarked. Any double named otherwise is invisible.
3. **Decide `ratatui-testlib`'s harness layer.** Leaning no on architecture: launch must happen inside bwrap and the PTY runner accepts only a prepared process, while `TuiTestHarness` owns spawning and ships no sandbox. Six weeks at v0.1.0 with one maintainer likely fails the well-maintained bar for a foundational layer.

**Held, not scheduled.** `jsonschema` 0.48.5 and `generic-array` 0.14.7 sit behind latest with 0 semver updates available. The `locked` policy holds them until a spec or a Captain decision moves them. No spec needs either. Revisit at next harbour.

**Open, needs the user.** Tier budgets. `budget: 120s` is a full-regression ceiling with no producer and no check: the `coverage` commands deliberately skip the weather append, so harbour's own regression records nothing. Observed weather gives real floors now, `@logic` ~24s, `@sandbox` ~26-34s, `@inference` ~63s with one 348s outlier during the flake. Setting a tier `budget` needs the user's word per the Verification agreement.

## The false-green pattern. The most expensive thing this project has learned.

Eight instances now, every one green while asserting nothing. The six older ones: a perturbation discharged by deleting its line; a `menu` role the deterministic pass already produced; a launch answering `ok:true` before the program started; a recorded plan carrying no expectation; two activation scenarios asserting text the fixture drew anyway; and a scantling that is valid JSON and an invalid schema.

Two more this harbour, both caught only by planting:

- **The gplint glob.** `features/**/*.feature` matches zero files in gplint's glob, because `**` must consume at least one directory segment and `features/` is flat. It exits 0 having linted nothing. Captain and Shipwright hit this independently, an hour apart, both reaching for the idiomatic form. The working values are `features/**` and `features/*.feature`. `RIGGING.md` carries the former and `AGENTS.md` records the trap.
- **A plank string shared across six seams.** `@planks("a process is prepared and launched")` is carried by `process.rs`, `sandbox.rs` three times, `bwrap.rs` and `pty.rs`. The string join is green no matter which of the six is dead, so a shared plank string makes the join structurally unable to see a dead seam. The trap is real; the seam it was found on turned out not to be dead. See the negative-control rule below.

**The test: when a scenario goes green early and cheaply, ask what it would take to make it red. If nothing would, the scenario is the defect.**

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
2. **An adoption proof leaves no durable trace.** A green looks identical proven or not. The proof-contract meta-schema and the gplint command were both proven red this harbour, in a Captain session and a Shipwright session; neither proof survives the context clear. A proof only its author can see is one clear from being unproven again.

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

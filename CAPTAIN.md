> STOP. Captain's notes: non-binding. Captain writes, Captain trims. Anyone else: close this file now.

# Captain Notes

Binding behaviour lives in `.feature` specs and referenced scantlings. History lives in git. These notes carry only what the next cycle needs.

## Deck state, 2026-07-26

Clean tree. Voyage 3 custody landed at `0ebc5df`, with notes commits on top. **The branch is ahead of `origin/main` and unpushed; that is the one live operator decision.** Confirm the count with `git rev-list --left-right --count origin/main...HEAD` rather than trusting a number written here. Tags `v0.1.1` at `1daee60` and `v0.1.2` at `9cc95b5`, both on origin. crates.io carries 0.1.0, 0.1.1 and 0.1.2, all by `dmytri`, none yanked.

Voyages 1 through 3 are closed and the watchbill is struck. All three tiers green, rerun fresh at custody: `@logic` 125 of 125 in 24.3s, `@sandbox` 31 of 31 in 26.4s, `@inference` 3 of 3 in 63.9s, all inside the 120s budget. 336 planks, 174 distinct, zero stale, malformed or provisional. Conformance, `fmt` and `clippy` clean. No perturbation stands.

0.1.2 is verified as a published artifact, not by eye: the `.crate` was downloaded from `static.crates.io` and carries all eleven scantlings with all thirteen URIs at `@v0.1.2`. Nothing under `src/` has changed since, so 0.1.2 remains the correct release and no new publish is owed.

## Scheduled work

Every item names its pivot and its role. An item with no pivot is backlog debt, so it is either scheduled here or struck. We do not carry "someday".

Next harbour, Shipwright:

1. **Derive `step-usage`.** The strongest item. Four roles in a row have discharged the plank join with an ad-hoc script, which is not repeatable custody. A checker extracting the `#[given]`, `#[when]` and `#[then]` pattern literals makes it a real project command and closes item 2 with it.
2. **Five orphaned step definitions in `tests/cucumber.rs`, and three planks in `src/inference.rs` naming three of them.** One fault, not two: those seams trace to a contract no scenario asserts, and the string join stays green because the definitions still exist. Behaviour-staleness, which no command on this stack reaches until item 1 lands.
3. **Two unplanked seams in `src/skill.rs`**, `FrontMatter` and `Skill`. Shipwright writes `@captain` skeletons or condemns them.
4. **Note the `focused` tag-exclusion exception in `RIGGING.md`.** cucumber-rs answers `error: the argument '--name <regex>' cannot be used with '--tags <tagexpr>'`, so this stack cannot satisfy the Rigging read contract on that one command. The value is correct as written and a role obeying the contract literally gets a hard error.
5. **Derive the scantling-enum join.** No check joins a scantling enum to the production enum it constrains, which is how the 17-role `Role` drift against the 11-role `tom.schema.json` went unseen.
6. **Shape-validate the three proof contracts.** `bwrap-isolation-policy`, `pty-sandbox-boundary` and `assistant-command-boundary` declare no dialect, so nothing validates their own shape: a typo'd key would be silently ignored by the checker reading it, leaving its attestation green. Identical to the false green watch1 closed for the other eight. Captain authors the meta-scantling; Shipwright derives the check.
7. **Decide `ratatui-testlib`'s harness layer**, per the prior-art section below. The emulator half is settled and shipped; only the harness question stands.

Next voyage, Captain:

8. **`cwd` is documented in `prepared-process.schema.json` and absent from `PreparedProcess`.** The conformance scenario is green because the field is optional, so this is latent divergence. `AGENTS.md` says the PTY runner accepts only a prepared process and never constructs backend arguments itself, which argues the field lands rather than the scantling drops it. Captain writes the scenario that drives it.

## The false-green pattern. The most expensive thing this project has learned.

Six of them across voyages 2 and 3, every one green while asserting nothing: a perturbation discharged by deleting its line; a `menu` role the deterministic pass already produced, so the engine's answer was never tested; a launch that answered `ok:true` when the program had not started, leaving every later step asserting against a blank screen; a recorded plan carrying no expectation; two activation scenarios asserting text the fixture drew in frame 1 regardless; and a scantling that is valid JSON and an invalid schema, so it validates everything.

**The test: when a scenario goes green early and cheaply, ask what it would take to make it red. If nothing would, the scenario is the defect.** Plant the red before trusting the green. QM did this for the activation fix and Captain did it for both watch1 checks; those are the scenarios worth having.

Two corollaries earned the hard way:

- **Where durable context changes and no scenario reddens, write the scenario that reddens.** Reach for a perturbation only where behaviour genuinely cannot be pinned. Watch ordering cannot substitute for a failing target: Crew struck a perturbation from inside the very watch meant to drive the work, because nothing in that watch required it.
- **A ruling that widens what one layer owes narrows what every other layer may claim.** Re-read every scenario that counts on the definition in the same pass. This was missed twice and QM found it both times.

## Rules of the language and the tooling

- **A `Background:` belongs to the `Rule:` it sits under.** Adding a `Rule:` to a feature that has a `Background:` is a structural edit, not a prose edit: it orphans the background for every scenario after it. This reddened seven scenarios once. No feature file now places a `Rule:` above its `Background:`; check before adding one.
- **Read the registry before naming a version.** A published version is immutable, so the check is cheap and the mistake is not. `cargo search tinman` answers it in one command and is already the `## Outbound` verify line; it works as a pre-flight read too. Recommending a `v0.1.1` tag without it cost a force-moved tag on a public remote.
- cucumber-rs makes `--tags` and `--name` mutually exclusive. Tag exclusion rides `CUCUMBER_FILTER_TAGS`; `--name` selects the scenario. Encoded in `RIGGING.md` `focused`.
- Runner is `tests/cucumber.rs`, `harness = false`, `fail_on_skipped()` so undefined steps redden.
- No clean cucumber-rs dry-run, so `discover: none`. A tag filter matching nothing reports `0 features` and proves nothing; to prove specs parse, run the default tier and read the feature count.
- Read the `result` field before trusting a weather line: `101` is a cargo test failure, so a fast line with `result: 101` is an early abort wearing a duration.
- `budget: 120s` on the default tier. `budget-sandbox` and `budget-inference` stay unset: no run in those tiers has yet produced a floor worth setting a ceiling from.
- Env confirmed: rustc 1.97 (edition 2024), bwrap 0.11.2, user namespaces enabled. `.env` holds a live `TINMAN_API_KEY`, git-ignored, so the `@inference` tier is runnable and costs money per run.

## Two doctrine gaps worth raising upstream

1. **A release version bump has no owner.** The write-scope list gives Shipwright the manifest for dependency install and upgrade only, and Shipwright is a harbour role while a release is mid-outbound. Captain bumped `Cargo.toml` to 0.1.2 under Captain's authority at sea, recorded as a departure. Boatswain independently named the same gap.
2. **An adoption proof leaves no durable trace.** The planted reds for both watch1 checks ran in Captain's session; the run record carries greens only, and a green looks identical proven or not. Boatswain correctly labelled the proof unverified because no command reaches it. A proof only its author can see is one context clear from being unproven again.

## Design decisions that bind (user-confirmed)

- **Tinman is a driver, not only a CLI.** Tests live in pytest, jest, bun test and drive Tinman the way Playwright's language clients drive the Node driver. `tinman driver` speaks newline-delimited JSON on stdin and stdout. This is the primary consumption surface; the YAML plan stays canonical for recorded flows. The protocol is RPC, not a second test format, so "no programming-language DSL" holds.
- **TOM is the DOM equivalent and inference is codegen.** The deterministic builder is the spine and produces every addressable role, including `menu`, `menuitem`, `button` and `textbox`, from terminal idioms. The LLM engine is a second producer of the same shape, capture time only, and proposes **names, never roles**. A hand-authored plan needs no model, which is why replay needs none.
- **Resolution and confirmation are two operations.** Resolution answers what a locator matches as the model stands and reports ambiguity as ambiguity, which is what a replaying test needs. Confirmation runs at capture time only, narrowing by scope or ordinal until one region binds. Collapsing them makes an ambiguous locator look bindable to the test that must later resolve it alone.
- **Terminal size is a property of the run, never of the plan.** The caller supplies it, defaulting to the operator's terminal, and it reaches the PTY and virtual screen together. A plan recording its capture size would invite replay to restore it.
- **Plan YAML grows with the test.** One canonical model, several surface forms. Shorthand removes typing, never adds capability, never weakens a default. An omitted `sandbox:` block means secure defaults, not no sandbox.
- **Help text is an asset, not a scenario.** Copy lives in `assets/help/`; scenarios own only the seams we own. No acronym validation: the generator gets the bundled skill's name and description and nothing else, and whatever comes back fills the tagline. Shaping the request is not validating the response.
- **Inference: any OpenAI-compatible provider, OpenRouter the default.** `TINMAN_API_KEY`, `TINMAN_BASE_URL` defaulting to `https://openrouter.ai/api/v1`, `TINMAN_MODEL` defaulting to `deepseek/deepseek-v4-flash`. Environment wins over `.env`. The credential is vendor-neutral by name so the default is never lock-in.
- **Tier placement.** `@sandbox` marks scenarios whose assertion is isolation itself; ordinary PTY launches stay default tier. `@inference` is real paid provider calls, never on the inner loop. If a fixture-launching default-tier scenario needs real bwrap, retag the scenario rather than weaken the tier policy.
- **Emulator is `alacritty_terminal` 0.26.0.** `vt100` was unmaintained and failed cell addressing and reverse video; it is gone from the graph. `wezterm-term` is the better emulator and is disqualified because it is not on crates.io and we publish. `alacritty_terminal` is 0.x and breaks across releases; budget for that.

## Standing preferences the user has stated

Read these before authoring any spec.

- **Prefer a concise attestation on a scantling over verbose behavioural scenarios**, and audit freshly authored specs against that, not only inherited ones. The tell is one rule restated once per variable: five `inference-provider` scenarios collapsed to two behaviour scenarios plus one `@contract` this way.
- **A scantling based on a well-known standard beats a bespoke one.** Applied: JSON-RPC 2.0 for the driver protocol, WAI-ARIA for the role taxonomy, JSON Schema 2020-12 throughout. **Prefer specifications over implementations** — ARIA, JSON-RPC, JSON Schema and AccName outlive any library. A well-maintained crate is a sound dependency for code and a poor foundation for a durable artifact, which is why AccessKit's node shape was declined.
- **`Rule:` prose carries durable context only.** Requirements belong in scenarios.
- **Tinman must stay useful for plain command-line testing**, not only full-screen work. A coding agent is both: one prompt non-interactively is a CLI, the same binary interactive is a TUI, and a suite driving it should need one tool. This is why a run step reads pipes and keeps its streams and exit status distinguishable.
- **No backlog debt.** Do it, decline it, or name its pivot: next watch, next voyage, next harbour.
- Breaking changes are acceptable at 0.1.x, so a correct reshape does not wait for a major version.
- **Specify the floor as well as the ceiling.** A scenario asserting a measurement against a threshold also carries a null control: an empty or no-op operation measures near zero. Executable specs make behaviour legible and calibration opaque, so a threshold scenario reads as a product claim while the step definition quietly binds a number no reader of the feature can see. The worked case is `ratatui-testlib`: `assert_render_budget(60.0)` asserts input-to-render latency under 16.67ms while the harness held an unconditional 50ms sleep and a 100ms read timeout, a measurement floor near 150ms, about nine times the threshold asserted. Add a metamorphic relation where one fits, such as ten keystrokes measuring about ten times one keystroke. Never instrument a latency measurement across a synchronization primitive we control: mark t0 at the write and t1 at the first response byte, never across our own poll loop. The same rule generalizes past latency: `all eight` and `all thirteen` in the watch1 checks are floors, so a glob reading nothing fails rather than passes.
- **Auto-memory is off**, `autoMemoryEnabled: false` in both `~/.claude/settings.json` and `.claude/settings.local.json`. A memory store re-injecting Captain decisions into role sessions makes the Context bulkhead advisory. Do not write memories on this project; anything worth keeping goes to a `.feature`, a scantling, `AGENTS.md`, `RIGGING.md`, or here.

## Prior art: ratatui-testlib, and the accessibility angle

`ratatui-testlib`, crates.io, MIT, first published 12 June 2026, v0.1.0, single maintainer `beengud` at `github.com/raibid-labs/ratatui-testlib`. Framework-neutral despite the name. It occupies Tinman's transport layer via the same `portable-pty`, and carries no semantic model, no sandboxing, no replay, no inference.

**Open, item 7 above.** Adopting its harness fights the architecture: launch must happen inside bwrap and the PTY runner accepts only a prepared process, while `TuiTestHarness` owns spawning and ships no sandbox. Leaning no, on architecture rather than maturity. User constraint is to adopt only if well maintained, which six weeks at v0.1.0 with one maintainer likely fails for a foundational layer.

Snapshot testing was considered and rejected: `insta` asserts "same as last time", which is drift detection rather than specification, and a snapshot passes whatever it recorded including a regression recorded before anyone looked.

**Accessibility is the TOM's second consumer.** Not scope and not a promise; a standing reason to hold the model honest and a positioning asset. Terminal accessibility is an empty category: VTE exposes a flat `AtkText`, Windows Terminal a UIA `TextPattern`, so an ncurses menu reaches a screen reader indistinguishable from a paragraph. Ratatui's a11y issue is open and unassigned. A black-box model deriving ARIA roles and accessible names from an attributed cell grid is the missing piece, and the TOM already emits that shape. Two consequences, both free: a wrong role passes a test but a listener notices at once, so holding the model to what a human would need spoken aloud improves it for testing; and never route the TOM through a lossy intermediate, because attribute-derived structure needs a genuine attributed grid and a terminal is the one place that hands you one. Map to AccessKit at a bridge if ever wanted; do not take its node shape into the scantling.

Keep the framework-neutral naming: `assets/skill/SKILL.md` says "CLIs and full-screen TUIs" and names no framework.

## Reference docs, not sanctioned artifacts

`idea.md`, `isolation.md` and `help.md` are intent sources in the crate's `exclude` list. Binding shape lives in specs, scantlings and `assets/**`. Do not treat them as requirements.

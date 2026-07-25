> STOP. Captain's notes: non-binding. Captain writes, Captain trims. Anyone else: close this file now.

# Captain Notes

Binding behaviour lives in `.feature` specs and referenced scantlings. History lives in git. These notes carry only what the next cycle needs.

## Prior art: ratatui-testlib

Checked 2026-07-25. `ratatui-testlib`, crates.io, MIT, first published 12 June 2026, v0.1.0 MVP, single maintainer `beengud` at `github.com/raibid-labs/ratatui-testlib`.

Despite the name it is framework-neutral: `ratatui` is an optional dependency behind a `ratatui-helpers` feature. Core dependencies are `portable-pty`, `termwiz`, `vtparse`. `TuiTestHarness` spawns any terminal program through `CommandBuilder` and asserts against the emulated screen.

It occupies Tinman's transport layer, and it uses the same `portable-pty` crate; the emulator differs, `termwiz`/`vtparse` against our `vt100`. What it does not carry: any semantic model, assertions are `text_at`, `cursor_position` and sixel bounds against raw cells; no sandboxing; no replay; no inference. So the commoditized part is plumbing Tinman would never have differentiated on, and the TOM, isolation-by-default and deterministic replay are untouched.

Two consequences. Not a dependency candidate: it ships no sandbox, which is a hard requirement here, and it would replace a layer we already have. And a positioning note: a crate named for one framework under-indexes for general TUI testing, so Tinman's framework-neutral naming is an advantage to keep. `assets/skill/SKILL.md` already says "CLIs and full-screen TUIs" and names no framework; keep it that way.

Snapshot testing was considered and rejected in the same pass. `insta` asserts "same as last time", which is drift detection rather than specification: a snapshot passes whatever it recorded, including a regression recorded before anyone looked. That contradicts the falsifiable-scenario line the project runs on. The existing `TestBackend` use at `tests/cucumber/support.rs` is the right depth.

## Voyage 2 — everything remaining from idea.md, isolation.md and help.md

User directive: take the whole remaining scope in one voyage, no backlog debt. 89 directed scenario targets across 8 watches, then the `@sandbox` and `@inference` tier sweeps as watch9 and watch10.

Voyage 1 shipped the capture spine: sandbox launch, PTY, virtual screen, Ratatui view, key recording, interaction log. Those 11 feature files are untouched and stay out of the watchbill.

Watch1 is now the three methodology-conformance scenarios. They gate the rest of the voyage: proving them first means every later watch runs under live plank-form, perturbation-quiescence and forbidden-doubles checks. QM owes each one a planted red at adoption, per the Verification policy.

## Design decisions this voyage (user-confirmed)

- **Tinman is a driver, not only a CLI.** Tests live in pytest, jest, bun test and drive Tinman the way Playwright's language clients drive the Node driver. `tinman driver` speaks newline-delimited JSON on stdin and stdout. This is the primary consumption surface; the YAML plan stays canonical for recorded flows. Keeps idea.md's "no programming-language DSL" intact, because the protocol is RPC, not a second test format.
- **TOM is the DOM equivalent and inference is codegen.** Deterministic builder is the spine: geometry from Ratatui-shaped nested rects, roles and names from heuristics. The LLM engine is a second producer of the same shape, capture time only. A hand-authored plan needs no model. This is why replay needs no model.
- **Plan YAML grows with the test.** One canonical model, several surface forms. Shorthand removes typing, never adds capability, never weakens a default. `features/plan-shorthand.feature` pins that both example assets parse identically, and that an omitted `sandbox:` block means secure defaults rather than no sandbox.
- **Help text is an asset, not a scenario.** Per the content policy: copy lives in `assets/help/`, scenarios own only the seams we own. `assets/help/tinman.txt` carries one `{{tagline}}` placeholder that inference fills; `inference-unavailable.txt` and `assistant-prompt.txt` carry the other two operator-visible strings. Inlined at build time.
- **No acronym validation. Settled 2026-07-25, user directive "don't over specify, let inference do whatever it wants give the name and description."** The generator receives the bundled skill's name and description and nothing else; whatever comes back fills the tagline. `features/acronym.feature` dropped from ten scenarios to two: the generated expansion fills the tagline, and an empty generation falls back to the unavailable notice. The six-word rule, the connective rule, the initials-spell-TINMAN check, the punctuation and newline rejections and the retry-once behaviour are all gone. help.md's "exactly six words" is superseded and is no longer a live question.
- **Inference: any OpenAI-compatible provider, OpenRouter as the default. Settled 2026-07-25.** Three configuration values, environment or dotenv, environment winning: `TINMAN_API_KEY` for the credential, `TINMAN_BASE_URL` defaulting to `https://openrouter.ai/api/v1`, `TINMAN_MODEL` defaulting to `deepseek/deepseek-v4-flash`. The credential is vendor-neutral by name so the default choice is never a lock-in. `features/inference-provider.feature` pins both defaults and both overrides plus bearer-token construction. `ureq` for the call, blocking, keeps tokio a dev-dependency. `dotenvy` for `.env`, which is git-ignored.
- **Tier placement.** `@sandbox` marks scenarios whose assertion is isolation itself, matching voyage 1's line; ordinary PTY launches stay default tier. `@inference` is new: real paid provider calls, never on the inner loop. If QM finds a fixture-launching default-tier scenario needs real bwrap, retag it rather than weaken the tier policy.

## Deck state at this harbour review

Shipwright's harbour work sits uncommitted and is confirmed by command, not by recall. Both prior blockers are cleared in the tree: `ureq` and `dotenvy` are in `Cargo.toml`, and `help.md` joined the `exclude` list. Rigging gained working `coverage`, `plank-inventory` and `conformance` commands plus weather and runrecord paths. `ast-grep scan` exits 0 clean and `plank-inventory` returns real entries; both were run, not read.

Condemnation is processed. Shipwright removed both watchbill-shape scenarios; zero `@shipwright` and zero `@captain` remain, and `features/methodology-conformance.feature` now carries exactly the three rule-backed `@conformance` scenarios. Typecheck, lint and `ast-grep scan` all exited 0 after the removal, and the default tier parsed 32 features with 105 live scenarios, unchanged from before the removal.

Two things still open:

1. **Watchbill-shape conformance is deliberately absent.** It is one of the two checks Shipwright's derivation names as required. Condemning it means a malformed watchbill blocks QM at dispatch rather than reddening as a target. Recorded in `AGENTS.md` so a later harbour does not re-derive a settled decision.
2. **Harbour's full regression is unevidenced.** Shipwright's report died with its context, so no run backs the harbour work. Harbour's own regression is what harbour work rides outbound on. Re-run it before any outbound, or accept that the harbour edits ship on voyage evidence alone.

## Rigging quirks learned

- cucumber-rs makes `--tags` and `--name` mutually exclusive. Tag exclusion rides `CUCUMBER_FILTER_TAGS`; `--name` selects the scenario. Encoded in RIGGING.md `focused`.
- Runner is `tests/cucumber.rs`, `harness = false`, `fail_on_skipped()` so undefined steps redden.
- No clean cucumber-rs dry-run, so `discover: none`. A tag filter matching nothing reports `0 features` and proves nothing; to prove specs parse, run the default tier and read the feature count in the summary.
- Env confirmed: rustc 1.97 (edition 2024), bwrap 0.11.2, user namespaces enabled.

## Harbour findings carried

- `step-usage` stays `none`: cucumber-rs has no usage report, so the stale-plank join has no machine-readable source and plank staleness is still caught by reading. Closing it needs a checker extracting the `#[given]`, `#[when]` and `#[then]` pattern literals and joining them against the plank inventory. Recorded in AGENTS.md.
- `check_bwrap_policy`/`check_pty_boundary` read JSON scantlings via `serde_yaml`; may migrate to `serde_json` now it is present.
- No derived check joins a proof scantling to its attesting scenario. A path grep across the specs reports `pty-sandbox-boundary.json` and `assistant-command-boundary.json` unreferenced, because the attestation form names the seam rather than the path. Both do have attesting scenarios. Any future scantling-reference check must know the two forms apart, or it reddens on correct specs.
- idea.md, isolation.md and help.md are intent-source reference docs, not sanctioned artifacts. Binding shape lives in specs, scantlings and `assets/**`.

## Open questions

None standing. `.env` holds a live provider credential under `TINMAN_API_KEY`, so watch10, the `@inference` sweep, is runnable. An earlier note in this file claimed that key was empty; that was wrong, corrected by reading the file. `.env` is git-ignored and its value never travels into a spec, a report, or a commit.

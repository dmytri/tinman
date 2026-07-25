> STOP. Captain's notes: non-binding. Captain writes, Captain trims. Anyone else: close this file now.

# Captain Notes

Binding behaviour lives in `.feature` specs and referenced scantlings. History lives in git. These notes carry only what the next cycle needs.

## Voyage 2 — everything remaining from idea.md, isolation.md and help.md

User directive: take the whole remaining scope in one voyage, no backlog debt. 94 directed scenario targets across 7 watches, plus the `@sandbox` and `@inference` tier sweeps.

Voyage 1 shipped the capture spine: sandbox launch, PTY, virtual screen, Ratatui view, key recording, interaction log. Those 11 feature files are untouched and stay out of the watchbill.

## Design decisions this voyage (user-confirmed)

- **Tinman is a driver, not only a CLI.** Tests live in pytest, jest, bun test and drive Tinman the way Playwright's language clients drive the Node driver. `tinman driver` speaks newline-delimited JSON on stdin and stdout. This is the primary consumption surface; the YAML plan stays canonical for recorded flows. Keeps idea.md's "no programming-language DSL" intact, because the protocol is RPC, not a second test format.
- **TOM is the DOM equivalent and inference is codegen.** Deterministic builder is the spine: geometry from Ratatui-shaped nested rects, roles and names from heuristics. The LLM engine is a second producer of the same shape, capture time only. A hand-authored plan needs no model. This is why replay needs no model.
- **Plan YAML grows with the test.** One canonical model, several surface forms. Shorthand removes typing, never adds capability, never weakens a default. `features/plan-shorthand.feature` pins that both example assets parse identically, and that an omitted `sandbox:` block means secure defaults rather than no sandbox.
- **Help text is an asset, not a scenario.** Per the content policy: copy lives in `assets/help/`, scenarios own only the seams we own. `assets/help/tinman.txt` carries one `{{tagline}}` placeholder that inference fills; `inference-unavailable.txt` and `assistant-prompt.txt` carry the other two operator-visible strings. Inlined at build time.
- **Acronym validator: six acronym-bearing capitalized words, lowercase connectives permitted and ignored.** help.md says "exactly six words" but its own shipped example, `Terminal Inference for Navigating Model Agent Networks`, has seven. Strict-six would reject the user's own example at runtime. Both forms are pinned in `features/acronym.feature`. Revisit if the user prefers strict-six.
- **Inference: OpenRouter with DeepSeek, behind a provider trait.** `ureq` for the call, blocking, keeps tokio a dev-dependency. `dotenvy` for `.env`, which is git-ignored. Environment beats dotenv file.
- **Tier placement.** `@sandbox` marks scenarios whose assertion is isolation itself, matching voyage 1's line; ordinary PTY launches stay default tier. `@inference` is new: real paid provider calls, never on the inner loop. If QM finds a fixture-launching default-tier scenario needs real bwrap, retag it rather than weaken the tier policy.

## Blockers to clear before QM sails

Both are Shipwright's, both confirmed by command, neither resolvable by Captain.

1. `ureq` and `dotenvy` are recorded under `## Dependencies` but absent from `Cargo.toml`. Installation is Shipwright's per the Rigging read contract. Watch1 does not need them; watch2 onward does. Install before QM so the voyage does not block mid-watch.
2. `help.md` and `isolation.md` are now tracked as of `33ca946`, and `cargo package --list` shows `help.md` in the crate. `Cargo.toml` `exclude` lists `idea.md`, `isolation.md`, `CAPTAIN.md`, `.claude`, not `help.md`. Add it. Only bites at `cargo publish`, which is Captain-only outbound, so it is not urgent, but it is live rather than hypothetical.

## Rigging quirks learned

- cucumber-rs makes `--tags` and `--name` mutually exclusive. Tag exclusion rides `CUCUMBER_FILTER_TAGS`; `--name` selects the scenario. Encoded in RIGGING.md `focused`.
- Runner is `tests/cucumber.rs`, `harness = false`, `fail_on_skipped()` so undefined steps redden.
- No clean cucumber-rs dry-run, so `discover: none`. A tag filter matching nothing reports `0 features` and proves nothing; to prove specs parse, run the default tier and read the feature count in the summary.
- No Rust CLI for step-usage / plank-inventory; both `none`, plank checks land at first harbour.
- Env confirmed: rustc 1.97 (edition 2024), bwrap 0.11.2, user namespaces enabled.

## Harbour findings carried

- No derived plank check: `plank-inventory`/`step-usage`/`conformance` are `none`; Rust here has no docblock reader. Shipwright derives a docblock plank-form check, proven by a planted red.
- `check_bwrap_policy`/`check_pty_boundary` read JSON scantlings via `serde_yaml`; may migrate to `serde_json` now it is present.
- No derived check joins a proof scantling to its attesting scenario. A path grep across the specs reports `pty-sandbox-boundary.json` and `assistant-command-boundary.json` unreferenced, because the attestation form names the seam rather than the path. Both do have attesting scenarios. Any future scantling-reference check must know the two forms apart, or it reddens on correct specs.
- idea.md, isolation.md and help.md are intent-source reference docs, not sanctioned artifacts. Binding shape lives in specs, scantlings and `assets/**`.

## Open questions

- Strict-six acronym validation instead of the connective rule, if the user prefers it over their shipped example.
- `.env` currently holds an empty `OPENROUTER_API_KEY`. The `@inference` tier cannot run until the user supplies a key; every other watch is runnable without one.

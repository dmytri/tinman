> STOP. Captain's notes: non-binding. Captain writes, Captain trims. Anyone else: close this file now.

# Captain Notes

Binding behaviour lives in `.feature` specs and referenced scantlings. History lives in git. These notes carry only what the next cycle needs.

## Voyage 1 — full first milestone (sandboxed-by-default)

Scope confirmed with the user: the whole idea.md first milestone in one voyage, plus the isolation.md secure launch spine. Scantling-first per user directive.

In scope (watchbill order):
- Secure launch spine: SandboxSpec + PreparedProcess schemas, PTY-not-Bubblewrap boundary, bwrap-argv isolation policy, backend selection, CommandSpec parse.
- Capture: PTY runner takes a PreparedProcess, vt100 -> virtual Screen, Ratatui render (TestBackend).
- Recording + YAML: key recording (order + forwarding), screen snapshots, interaction-log YAML (schema + @contract).
- Real isolation (@sandbox): home/secret hidden, network denied.

Deferred (later voyages, user-confirmed exclusion): TOM inference, semantic capture, replay, harness/flow YAML, multi-process orchestration, macOS backend, `none` backend beyond the dev/debug path the selection scenarios pin.

## Rigging quirks learned

- cucumber-rs makes `--tags` and `--name` mutually exclusive. Tag exclusion rides `CUCUMBER_FILTER_TAGS`; `--name` selects the scenario. Encoded in RIGGING.md `focused`.
- Runner is `tests/cucumber.rs`, `harness = false`, `fail_on_skipped()` so undefined steps redden.
- No clean cucumber-rs dry-run, so `discover: none`; undefined steps surface in focused runs.
- No Rust CLI for step-usage / plank-inventory; both `none`, plank checks land at first harbour.
- Env confirmed: rustc 1.97 (edition 2024), bwrap 0.11.2, user namespaces enabled. @sandbox tier is runnable in this VM.

## Voyage 1 progress

- QM run 1: watch1 7/9 green (gates clean), returned two fitting-out blockers.
  - Fixed `focused` (dropped `--tags`; cucumber-rs forbids `--tags`+`--name`; name-anchoring excludes skeletons). Proven green.
  - Installed `serde_json` + `jsonschema` (dev) for the schema `@contract` scenarios. Decision: standard Rust validator; dev-only. `cargo check` clean.
  - Refit committed as new base; watch1 code + specs remain work-in-flight to Boatswain.
- Next: fresh QM resumes the 2 schema scenarios (Crew must expand `SandboxSpec`/`PreparedProcess` to full serde shapes), then watch2 (PTY capture, Ratatui render, recording, YAML) and watch3 (@sandbox isolation).

## Harbour findings (from QM run 1, deferred to first harbour)

- No derived plank check: `plank-inventory`/`step-usage`/`conformance` are `none`; Rust here has no docblock reader. Shipwright derives a docblock plank-form check, proven by a planted red.
- `check_bwrap_policy`/`check_pty_boundary` read JSON scantlings via `serde_yaml`; may migrate to `serde_json` now it is present.
- `src/main.rs` still the base stub; no scenario drives the binary end-to-end (command-invocation asserts the `parse_command_line` seam). Candidate end-to-end scenario for a later voyage.
- idea.md / isolation.md are intent-source reference docs, not sanctioned artifacts; binding shape lives in specs + scantlings. Fold any still-durable constraint into `Rule:` prose if a gap appears.

## Rename decision (Clanker -> Tinman)

- User renamed the project (too many tools called "clanker"). Directory moved to `~/tinman` by the user; crate/binary/code/specs/scantlings/docs/memory renamed by Captain.
- A global identifier rename spans Crew (`src/`) and QM (`tests/`) scopes; no single role cleanly owns it. Taken as Captain minimal-action-to-restore-progress: behaviour-preserving textual refactor of uncommitted in-flight work, re-proven by `cargo check` + a re-run watch scenario, and the re-dispatched QM re-verifies from scratch.
- Settings allowlist absolute path updated to `~/tinman`. Memory relocated to the `-home-exedev-tinman` keyed store for future sessions.
- An orphan Crew from the killed QM landed watch2's first 3 scenarios (virtual-screen x2, terminal-view) green under tinman; kept as work-in-flight.

## TOM roadmap (design constraint, user-confirmed)

- Voyage 1 capture = whole virtual screen (full PTY grid), no pane/role, no TOM. User confirmed this is fine "as long as we plan to make it TOM-bindable later."
- `VirtualScreen` is the binding substrate and MUST keep the full cell grid (position + content); the `cell at row/col` scenarios protect that surface. A later voyage adds TOM inference (nested rects + roles) over `VirtualScreen`, then the semantic `capture: {role, items, scope}` op and webrat-style role binding.
- Do not collapse `VirtualScreen` to a flat string or drop cell positions; that would break future TOM binding.

## Open questions

- None blocking. macOS backend and the full harness/sandbox YAML parser are deferred by design.

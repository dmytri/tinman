# Tinman

A deterministic black-box testing framework for CLIs and full-screen TUIs —
"webrat/capybara/selenium for terminals." Tinman drives real terminal programs,
including real coding agents, through an embedded PTY, and never inspects
application internals.

- **Capture time** may infer a mechanical test plan.
- **Replay time** is absolutely deterministic: no model invocation, no network.

Sandboxed execution is the default. On Linux, Tinman launches its target inside
a Bubblewrap sandbox; the operator's home, environment, and network are hidden
unless explicitly granted.

> **Status: early prototype.** This release reserves the name and ships the
> capture pipeline seams (PTY capture into a virtual screen, Ratatui rendering,
> key recording, and a YAML interaction log) under a spec-driven test harness.
> The end-to-end `tinman record` binary, the Terminal Object Model (TOM), and
> deterministic replay are in progress.

## License

Licensed under either of MIT or Apache-2.0 at your option.

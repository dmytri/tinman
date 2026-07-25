# Tinman

Tinman is a deterministic black-box testing framework for CLIs and full-screen TUIs. It drives real terminal programs, including real coding agents, through an embedded PTY and never inspects application internals. Capture time may infer; replay time is deterministic with no model invocation and no network.

## Method

This project uses **Shipshape**, a spec-driven, context-isolated workflow. Binding product behaviour lives in `.feature` specs under `features/`. Mechanical shape lives in scantlings under `scantlings/`. Tooling values a role reads on open live in `RIGGING.md`.

- Specifications are durable. Production code under `src/` is disposable from the specs.
- Verification is our dev rigging: cucumber-rs, run as a `cargo test` binary. It is real by default and exercises real Tinman seams. This real-by-default rule governs how we test Tinman; it is distinct from Tinman's own mandate, which is to drive real TUIs.

## Isolation

Sandboxed execution is the default. `tinman record` launches its target inside a sandbox; the only Linux backend is Bubblewrap. Unsandboxed execution is a hard failure unless an explicit unsafe option is set. The operator's real home, environment, and PATH are never inherited by default. The PTY runner accepts only a prepared process and never constructs backend arguments itself.

## Verification tiers

- Default tier (`@logic`, untagged): pure, local, deterministic. No external tool.
- `@sandbox` tier: launches a real process under Bubblewrap. Requires the `bwrap` binary and unprivileged user namespaces.

See `RIGGING.md` for the exact commands. Note the cucumber-rs constraint: `--tags` and `--name` are mutually exclusive on the CLI. A focused run selects one scenario with `--name "^…$"`, whose anchoring already excludes the differently-named `@captain` and `@shipwright` skeletons, so it carries no tag filter. Tier enumeration sweeps use no `--name`, so they carry the exclusion through the `CUCUMBER_FILTER_TAGS` environment variable.

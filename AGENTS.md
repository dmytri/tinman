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
- `@inference` tier: calls the configured inference provider for real. Requires `TINMAN_API_KEY`, read from the environment or from a git-ignored `.env` file. `TINMAN_BASE_URL` and `TINMAN_MODEL` are optional overrides, defaulting to OpenRouter and `deepseek/deepseek-v4-flash`. Tinman speaks the OpenAI-compatible chat-completions protocol, so any compatible endpoint serves. It costs money per run and never sits on the inner loop.

## Run data

The wake carries two records, both git-ignored under `target/` and both named in `RIGGING.md` under `## Tiers`.

`target/tinman-runrecord.jsonl` is the voyage run record. A role appends one line after a fresh green, in the shape the Transient output policy fixes.

`target/tinman-weather.jsonl` is yesterday's weather. Each tier enumeration sweep appends one line, and the `broad`, `broad-sandbox` and `broad-inference` commands carry that append themselves, so the record is produced by running the sweep and needs no runner support:

```json
{"tier":"@sandbox","workers":4,"ms":424,"result":101}
```

`result` is the sweep's exit status, so a reader tells a green worker count from a red one. The `coverage` commands deliberately do not append: `cargo llvm-cov` instruments the build, and its wall clock is not the prior a later uninstrumented sweep should start from.

Worker counts are derived per tier from that tier's binding constraint and are passed explicitly with `-c`, so the recorded count is a fact rather than the cucumber-rs default of 64. The default tier is local and pure, and runs at 64. The `@sandbox` tier spawns a real Bubblewrap process and PTY per scenario, so it is bound by local compute and runs at 4, one per core. The `@inference` tier is bound by the provider's rate limits and by cost, and runs at 2. Raise a count only on headroom this record confirms.

Two facts cucumber-rs does not give us: it emits no per-scenario duration, so weather is per-tier only, and it emits no structured pressure signal, so rate-limit and memory pressure are read from the sweep's own output rather than from a recorded field. Closing either needs a custom cucumber-rs `Writer`, which is verification support and QM's to write.

## Methodology checks

Methodology breaches surface as failing verification rather than as review comments. The rule set lives in `scantlings/verification-conformance` and is discharged by the `conformance` command, `ast-grep scan`, configured by `sgconfig.yml`. It carries three rules: plank form, perturbation quiescence, and forbidden doubles. The scenarios that run it are tagged `@conformance` in `features/methodology-conformance.feature`.

Watchbill-shape conformance is deliberately absent. Shipwright derived it and Captain condemned it at the 2026-07-25 harbour, on the decision that the watchbill stays hand-checked rather than schema-backed. A later harbour that re-derives it is repeating a settled decision, not finding a gap.

Two derivations the stack does not support: cucumber-rs offers no dry-run and no usage report, so `discover` and `step-usage` read `none` in `RIGGING.md`. The consequence is that the stale-plank join of plank strings against current step-definition patterns has no machine-readable source, and plank staleness is caught by reading rather than by running. Closing it needs a checker that extracts the `#[given]`, `#[when]` and `#[then]` pattern literals and joins them against the plank inventory.

See `RIGGING.md` for the exact commands. Note the cucumber-rs constraint: `--tags` and `--name` are mutually exclusive on the CLI. A focused run selects one scenario with `--name "^…$"`, whose anchoring already excludes the differently-named `@captain` and `@shipwright` skeletons, so it carries no tag filter. Tier enumeration sweeps use no `--name`, so they carry the exclusion through the `CUCUMBER_FILTER_TAGS` environment variable.

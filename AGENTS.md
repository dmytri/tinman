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

Methodology breaches surface as failing verification rather than as review comments. The rule set lives in `scantlings/verification-conformance` and is discharged by the `conformance` command, `ast-grep scan`, configured by `sgconfig.yml`. It carries four rules: plank form, plank presence, perturbation quiescence, and forbidden doubles. The scenarios that run it are tagged `@conformance` in `features/methodology-conformance.feature`.

Watchbill-shape conformance is deliberately absent. Shipwright derived it and Captain condemned it at the 2026-07-25 harbour, on the decision that the watchbill stays hand-checked rather than schema-backed. A later harbour that re-derives it is repeating a settled decision, not finding a gap.

`discover` reads `none`: cucumber-rs offers no dry-run form, confirmed against the runner's own `--help`. That absence has a consequence for role ordering, so sail it this way.

Doctrine's red-first flow assumes a dry-run exists to list unimplemented steps without building anything. Here there is none, so a step naming a production seam that does not exist yet fails at compile time rather than at run time. **That compile failure is a production-code failure and is legitimate evidence for a Crew dispatch** - the message names the absent seam as precisely as any assertion diff. A role may leave the crate uncompilable *during* its turn while producing that evidence. It must not *end* its turn there: the hand-off has to leave a tree custody can verify, so restore compilation before reporting. Where a scenario can observe the shipped binary instead of an internal seam, prefer that; it fails at run time and needs none of this.

`lint` chains feature lint before code lint: `npx --no-install gplint "features/**"`, then `cargo fmt --check` and `cargo clippy`. gplint is an npm dev dependency, so the rigging carries a `package.json` and `package-lock.json` beside `Cargo.toml`, and `RIGGING.md` records `packageManager` twice. `--no-install` honours the `locked` dependency policy: it resolves the lockfile's version and refuses to fetch a floating one.

The feature-file argument is `features/**`, and the obvious `features/**/*.feature` is wrong here. gplint's glob requires `**` to match at least one directory segment, so against a flat `features/` it matches zero files, lints nothing and exits 0. That is a silent false green: the gate passes because it read nothing. `features/**` matches both flat and nested files, and gplint filters the non-feature files itself. Prove any change to this argument by planting a violation and confirming the command reddens; a green run cannot tell a clean spec set from an unread one. The proof at the 2026-07-26 harbour planted a disallowed tag in the first and last feature files and confirmed a red on each, with gplint's JSON output reporting 34 files read against 34 on disk.

`step-usage` is derived from the step-definition source rather than from the runner, which reports no usage. It is an `ast-grep` scan over `tests` that captures the pattern literal of every `#[given]`, `#[when]` and `#[then]` attribute, in both the `expr = "…"` and the bare-literal form, and reports it untruncated as an ast-grep metavariable. The reported string is exactly the plank string the Planking agreement fixes, so the stale-plank join is now an exact-string set membership that a run decides.

That derivation covers the pattern side of the trace only. The last hop, which scenarios bind each pattern, stays underivable: cucumber-rs emits no usage report, and joining patterns to scenario steps means compiling Cucumber Expressions, which is checker logic and belongs in a step definition rather than in a command value. Two checks therefore need that join in their steps: the stale-plank join, and the orphaned-step-definition check that reddens a pattern no scenario binds.

Two derived checks report a known weakness, per the Check tooling rule. `plank-inventory` and the `plank-form` rule match a `line_comment` carrying `@planks`, so they see the `///` shape but cannot read what the comment says; rustc's own rule that a doc comment must attach to an item, with clippy run at `-D warnings`, is what makes the placement half of plank form executable. The `forbidden-doubles` rule keys on a type name matching `Mock`, `Fake`, `Stub` or `Dummy`, so a double named anything else is invisible to it: `LocalProvider` in verification support is a real double, correctly marked `@exceptional-double`, that the rule would not have reddened unmarked. Closing that needs a rule keyed on the double's shape rather than its name.

A coverage blind spot the summary does not announce: a scenario that drives `tinman` as a child process and then SIGKILLs it loses that child's coverage entirely, because a killed instrumented process never flushes its counters. `src/driver.rs` therefore reads 0.00% across every tier while the 24 scenarios binding its planks pass, and `fn main()` reads 0 executions in a run where seven driver scenarios are green. Read that 0% as unattributed, never as unreached: judge reachability from the import and call graph, per the "Current design only" Article.

See `RIGGING.md` for the exact commands. Note the cucumber-rs constraint: `--name` and `--tags` are mutually exclusive, and the exclusion reaches the environment variable too, because `CUCUMBER_FILTER_TAGS` is that same `--tags` argument. A run passing both fails with `the argument '--name <regex>' cannot be used with '--tags <tagexpr>'`, whichever route the tag expression arrives by. So `focused` genuinely cannot carry the tag exclusion the Rigging read contract asks of every verification command, and no rewrite of the value closes it.

What stands in for the exclusion is the anchoring: `--name "^…$"` selects by exact scenario name, so a `@captain` or `@shipwright` skeleton is excluded by carrying a different name. That substitute rests on two properties of the specs, and a scenario name is a regex here, not a literal. A name repeated inside one feature file would run both scenarios, condemned or skeleton included, and a name carrying a regex metacharacter would match the wrong scenario or none. Both properties hold today and nothing enforces either, so they are the subject of a derived `@conformance` check. Tier enumeration sweeps use no `--name`, so they carry the exclusion through `CUCUMBER_FILTER_TAGS` as normal.

## Releases

**Captain owns the release version bump, and it is one coupled edit rather than a field.**

Doctrine gives the bump no owner, so this project closes the gap locally. Shipwright holds the package manifests for dependency work and is a harbour role, while a release is mid-outbound. Crew is dispatched only for a failing target, and a bump has none. Boatswain writes hygiene rather than new content. The gap stays open upstream, so a release that waits for the rule to be worked out again waits every time.

These move together or not at all:

- `version` in `Cargo.toml`
- `version` in `npm/package.json`
- every published schema `$id` URI, nineteen as of 2026-07-29, across `scantlings/` and `assets/examples/`

Two limits on what the checks around this can see, both paid for on 2026-07-29.

`every published schema URI names the packaged version` compares the URIs against `Cargo.toml`, so it catches a forgotten URI and never a changed contract under an unchanged version. A schema whose content moves while the version stands leaves every URI still naming `@vX.Y.Z`, and the check stays green while that pinned URI serves something the repository no longer contains. The tree is in that state now: `sandbox-spec.schema.json` lost a required field with `Cargo.toml` still reading 0.2.0. The URI is pinned to a tag rather than to a branch, so publishing the next version is what resolves it, and a consumer reading the pinned URI meanwhile gets the older contract.

`both packaged manifests name one version` is newly authored and on the watchbill. Before it nothing joined the two manifests at all, so a bump applied to one could reach a registry naming a version that described different contents.

## Outbound

Tinman ships two targets, and they release independently. `RIGGING.md` carries the exact `ship` and `verify` commands for each under `## Outbound`.

**crates.io** ships the source crate. `cargo publish` from the repository root is the whole runbook.

**npm** ships `@dk/tinman`, a prebuilt `linux`/`x64` release binary rather than the source tree. Its manifest is durable at `npm/package.json` and everything beside it is staged at ship time, so the staged paths are git-ignored: the `ship` command builds the release binary, installs it at `npm/bin/tinman`, copies `README.md` and `LICENSE` into `npm/`, and publishes that directory. Those four files are the published tarball, because `files` names `bin` and npm adds the manifest, the readme and the licence itself. The package was first published ad hoc from outside the repository at 0.2.0 on 2026-07-28; the manifest here reproduces what that publish shipped.

The release profile sets `strip = true`, and that setting exists for this target. The binary is the package, and the intended invocation is the unversioned `npx @dk/tinman`, so anyone who has not pinned fetches it again on every run and debug symbols are weight none of them can use. Stripping took the binary from 7,221,104 to 5,610,464 bytes, the tarball from 2.7 MB to 2.5 MB, and the unpacked package from 7.2 MB to 5.6 MB. Verification is unaffected, because the scenarios drive the test-profile binary through `CARGO_BIN_EXE_tinman` rather than the release one.

Two versions have to move together and nothing yet joins them: `version` in `Cargo.toml` and `version` in `npm/package.json`. The schema-URI conformance scenario already reads the packaged version from `Cargo.toml` alone, so a bumped crate and a forgotten npm manifest is a drift no check reports.

**An outbound `verify` line installs and runs the artifact a user would receive. A registry lookup is not a verification.** A lookup answers whether a name resolves, which is a question nobody was asking; it says nothing about what the resolved artifact contains or whether it runs. Both lines here download the published package, install it, and execute the shipped binary, per the Outbound verification policy.

Each line also names the version it expects rather than accepting whichever one answers. The crates.io line reads that version from `Cargo.toml`, so it reddens when the registry is behind the repository.

This is not hypothetical. The previous line was `cargo search tinman`, which exits 0 whatever version comes back. The 0.2.0 release reached npm on 2026-07-28 and never reached crates.io, where 0.1.2 still stands, and `cargo search` reported success for a day across that gap. The replacement caught it on its first run.

## Settled decisions

Decisions a later harbour would otherwise re-derive from repository signals. A finding recorded only in Captain's private notes is invisible to every other role, so it is re-derived every harbour and re-argued every time; this section is where such a decision becomes durable. The watchbill-shape condemnation is recorded under Methodology checks above, in the same spirit.

`pty::launch` and `pty::capture` are a settled keep. Reference analysis shows both have no production caller: every launch path reaches the PTY through `capture_interactive` or `capture_interactive_at`. They were kept at an earlier harbour on the negative-control argument, and the 2026-07-29 harbour re-derived them as unreachable because that decision lived only in Captain's notes. They are not dead code and are not a finding. A harbour that reports them again is repeating a settled decision.

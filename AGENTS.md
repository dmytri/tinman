# Tinman

Tinman is a deterministic black-box testing framework for CLIs and full-screen TUIs. It drives real terminal programs, including real coding agents, through an embedded PTY and never inspects application internals. Capture time may infer; replay time is deterministic with no model invocation and no network.

## Method

This project uses **Shipshape**, a spec-driven, context-isolated workflow. Binding product behaviour lives in `.feature` specs under `features/`. Mechanical shape lives in scantlings under `scantlings/`. Tooling values a role reads on open live in `RIGGING.md`.

- Specifications are durable. Production code under `src/` is disposable from the specs.
- Verification is our dev rigging: cucumber-rs, run as a `cargo test` binary. It is real by default and exercises real Tinman seams. This real-by-default rule governs how we test Tinman; it is distinct from Tinman's own mandate, which is to drive real TUIs.

### Writing durable prose

**Durable prose never references watchbill membership.** Read this before writing a doc line beside a new scenario, which is where the temptation lands.

A watchbill is transient by design: custody strikes it the moment the voyage closes. A sentence saying a scenario is on the watchbill therefore begins decaying at that instant, and it decays silently, because nothing about the prose changes when the file disappears. The reader meets a confident claim with nothing left to contradict it. This file has already carried that drift three times, most recently on 2026-07-31, in a paragraph written after this rule was.

That last instance is the instructive one, so treat this rule as hard rather than obvious. It is easy to agree with in the abstract and hard to obey in the sentence, because the temptation arrives at one specific moment: you are documenting a check that does not exist yet, and you want to say it is coming. "On the watchbill" is the nearest handle for scheduled, and it is exactly the handle that rots. There is no durable way to write that something is coming. Write what is true of the tree now, and leave the landing for the writer who can see it.

Record that the scenario exists and what it asserts. How it was scheduled is history, and history lives in git.

Dated records of what was decided when are durable and wanted: "condemned at the 2026-07-25 harbour" stays true forever. It is only the claim that something is *currently* scheduled that rots. The test is whether the sentence is still true after custody closes the voyage.

**Durable prose names the check that carries a count rather than restating the number.** Same failure, one step further out.

A hardcoded count decays exactly as a watchbill reference does, and silently for the same reason: nothing about the sentence changes when a scantling is added, so it goes quiet rather than wrong. This file has carried a stale schema-URI count at seven, at eight, at nine and at nineteen. Each was true when written.

The count already has an owner, and it is the scenario. It is checked on every run and cannot drift without reddening, which is a guarantee prose can never offer. So point at it: "joined to the packaged version by `features/methodology-conformance.feature:every published schema URI names the packaged version`" stays true at any count, while "nineteen" is true only until the next voyage adds one.

The test is the same one, applied further out: is the sentence still true after the next voyage? A measurement with a date is exempt, because it is a record rather than a claim about now: "7,221,104 bytes on 2026-07-29" stays true forever. So is a small enumeration the sentence itself spells out, where the reader can count the items and see the number is right.

**Durable prose describes what a check decides, and cites the durable artifact rather than the machinery.** The same failure once more, at the mechanism.

A sentence that identifies a check by the machinery behind it decays every time that machinery moves, and it moves whenever the check gets better. The doubles paragraph under Methodology checks was rewritten three times on 2026-07-31: for a claim about what did not exist yet, for a watchbill citation, for a scenario title that changed, and for a function that was deleted. Every version was accurate the day it was written, and each was made false by an improvement rather than by a mistake.

This rule and the count rule above it look opposed, and they are not. The count rule says point at the scenario; this one says a name decayed. The line between them is this project's own premise, that specifications are durable and everything beneath them is disposable. A scenario title is a durable artifact, so citing one cites the contract. A function, a step definition, a rule file and a watchbill entry are all disposable, so citing one cites machinery that is expected to move. The doubles paragraph rotted because it named machinery, not because it pointed at a spec. Cite the durable thing, never the disposable one.

A citation names the file and the title together, the form the watchbill already uses: `features/methodology-conformance.feature:the verification-conformance rule set reports no match`. A bare title in backticks cannot be told from any other backticked string, so anything reading these documents has to guess which is which, and a survey of the shipped Markdown pulled `npm publish npm --access public` alongside real titles. The reference form announces itself and resolves exactly. A sentence is free to quote the title for a human beside it, and where a sentence reads better naming only the feature file, that is a durable pointer too and needs no title.

What a check decides is the durable part. It is the reason the check exists, and it survives rewrites of the mechanism that a name cannot. The test is the others' test, aimed at the next improvement rather than the next voyage: is the sentence still true after somebody makes this check better?

## Isolation

Sandboxed execution is the default. `tinman record` launches its target inside a sandbox; the only Linux backend is Bubblewrap. Unsandboxed execution is a hard failure unless an explicit unsafe option is set. The operator's real home, environment, and PATH are never inherited by default. The PTY runner accepts only a prepared process and never constructs backend arguments itself.

## Verification tiers

- Default tier (`@logic`, untagged): pure, local, deterministic. No external tool.
- `@sandbox` tier: launches a real process under Bubblewrap. Requires the `bwrap` binary and unprivileged user namespaces.
- `@inference` tier: calls the configured inference provider for real. Requires `TINMAN_API_KEY`, read from the environment or from a git-ignored `.env` file. `TINMAN_BASE_URL` and `TINMAN_MODEL` are optional overrides, defaulting to OpenRouter and `deepseek/deepseek-v4-flash`. Tinman speaks the OpenAI-compatible chat-completions protocol, so any compatible endpoint serves. It costs money per run and never sits on the inner loop.

## Run data

The wake carries three records, all git-ignored under `target/` and all named in `RIGGING.md` under `## Tiers`.

`target/tinman-runrecord.jsonl` is the voyage run record. A role appends one line after a fresh green, in the shape the Transient output policy fixes.

`target/tinman-weather.jsonl` is yesterday's weather. Each tier enumeration sweep appends one line, and the `broad`, `broad-sandbox` and `broad-inference` commands carry that append themselves, so the record is produced by running the sweep and needs no runner support:

```json
{"tier":"@sandbox","workers":4,"ms":424,"result":101}
```

`result` is the sweep's exit status, so a reader tells a green worker count from a red one. The `coverage` commands deliberately do not append: `cargo llvm-cov` instruments the build, and its wall clock is not the prior a later uninstrumented sweep should start from.

`target/tinman-durations.jsonl` is the per-scenario duration record, and it carries the attribution the weather record cannot. A sweep appends one line as each scenario starts, one carrying that scenario's own wall clock as it ends, and one as the run itself reaches its end:

```json
{"ms":230,"run":"2490178-1785703080650","scenario":"/home/exedev/tinman/features/driver-protocol.feature:70:the driver exits when its stdin closes"}
```

A scenario is keyed by the spec carrying it, the line it sits on, and its name, because a `Scenario Outline` expands to one scenario per example row and every row shares the outline's name. The spec is the path the runner reports, which is absolute, so joining this record to a watchbill reference means relativising it first. `run` tells one sweep's entries from another's in an append-only record, and it carries the moment the process started beside its pid, because a pid repeats on a long-lived machine. The start lines and the completion line are what let a reader tell a sweep that finished from one that was killed part way: a scenario that started and was never timed reads as a gap, where an unrecorded start would read as nothing at all. `features/methodology-conformance.feature:the wake records how long each scenario took` is the check that reads this record.

Worker counts are derived per tier from that tier's binding constraint and are passed explicitly with `-c`, so the recorded count is a fact rather than the cucumber-rs default of 64. The default tier is local and pure, and runs at 64. The `@sandbox` tier spawns a real Bubblewrap process and PTY per scenario, so it is bound by local compute and runs at 4, one per core. The `@inference` tier is bound by the provider's rate limits and by cost, and runs at 2. Raise a count only on headroom this record confirms.

Weather is per-tier and the durations record is per-scenario, and the split follows from where each is produced rather than from a limit of the runner. A sweep command wraps the whole run, so it can time the tier and nothing finer. The per-scenario clock is taken by the before and after hooks in `tests/cucumber.rs`, which the runner calls around each scenario, and the run is closed by hand rather than by `run_and_exit` so the completion line lands before a red is raised. A sweep that ends red still says where its time went.

cucumber-rs also ships a `JUnit` writer that records each scenario's own wall clock and a `Json` writer that records each step's duration, behind the crate's `output-junit` and `output-json` features, each of which turns on its `timestamps` feature. This project enables neither, and the hooks reach the same per-scenario measurement without them: `output-junit` pulls `junit-report`, and `output-json` pulls `base64`, `Inflector` and `mime`. Read a later proposal to enable one as a trade of dependencies for step-level timing, which is finer than anything here reads.

A structured pressure signal is the fact the runner genuinely does not carry: rate-limit and memory pressure are read from the sweep's own output rather than from a recorded field, and closing that does need a custom `Writer`.

## Methodology checks

Methodology breaches surface as failing verification rather than as review comments. The rule set lives in `scantlings/verification-conformance` and is discharged by the `conformance` command, `ast-grep scan` over that rule set, configured by `sgconfig.yml`, which prints its status before propagating it. The duplication scanner is not a leg of this command. It gates on `scantlings/duplication-allowance.json`, which names the groups that are structural coincidence, and an allowance is a list rather than a count: a command leg can only threshold on a number, so it would go green the day one real duplication is removed and another added. The join by fingerprint needs a reader of the allowance, so the scanner runs from the step definition behind `features/methodology-conformance.feature:every duplicate group the scanner reports is named as coincidence`, which reads it. That keeps one rule in one checker. The rule set carries the plank-form, plank-presence, perturbation-quiescence, process-wide-env-mutation, killed-measured-child and unshared-corpus-read rules. The floor is the step of that name in `features/methodology-conformance.feature:the verification-conformance rule set reports no match`, which joins the rule ids read from the directories `sgconfig.yml` names against the rules the step lists; read the floor from that step, which this sentence follows rather than fixes. The scenarios that run it are tagged `@conformance` in `features/methodology-conformance.feature`.

Watchbill-shape conformance is deliberately absent. Shipwright derived it and Captain condemned it at the 2026-07-25 harbour, on the decision that the watchbill stays hand-checked rather than schema-backed. A later harbour that re-derives it is repeating a settled decision, not finding a gap.

`discover` reads `none`: cucumber-rs offers no dry-run form, confirmed against the runner's own `--help`. That absence has a consequence for role ordering, so sail it this way.

Doctrine's red-first flow assumes a dry-run exists to list unimplemented steps without building anything. Here there is none, so a step naming a production seam that does not exist yet fails at compile time rather than at run time. **That compile failure is a production-code failure and is legitimate evidence for a Crew dispatch** - the message names the absent seam as precisely as any assertion diff. A role may leave the crate uncompilable *during* its turn while producing that evidence. It must not *end* its turn there: the hand-off has to leave a tree custody can verify, so restore compilation before reporting. Where a scenario can observe the shipped binary instead of an internal seam, prefer that; it fails at run time and needs none of this.

`lint` chains feature lint before code lint: `npx --no-install gplint "features/**"`, then `cargo fmt --check` and `cargo clippy`. gplint is an npm dev dependency, so the rigging carries a `package.json` and `package-lock.json` beside `Cargo.toml`, and `RIGGING.md` records `packageManager` twice. `--no-install` honours the `locked` dependency policy: it resolves the lockfile's version and refuses to fetch a floating one.

The feature-file argument is `features/**`, and the obvious `features/**/*.feature` is wrong here. gplint's glob requires `**` to match at least one directory segment, so against a flat `features/` it matches zero files, lints nothing and exits 0. That is a silent false green: the gate passes because it read nothing. `features/**` matches both flat and nested files, and gplint filters the non-feature files itself. Prove any change to this argument by planting a violation and confirming the command reddens; a green run cannot tell a clean spec set from an unread one. The proof at the 2026-07-26 harbour planted a disallowed tag in the first and last feature files and confirmed a red on each, with gplint's JSON output reporting 34 files read against 34 on disk.

`step-usage` is derived from the step-definition source rather than from the runner, which reports no usage. It is an `ast-grep` scan over `tests` that captures the pattern literal of every `#[given]`, `#[when]` and `#[then]` attribute, in both the `expr = "…"` and the bare-literal form, and reports it untruncated as an ast-grep metavariable. The reported string is exactly the plank string the Planking agreement fixes, so the stale-plank join is now an exact-string set membership that a run decides.

That derivation covers the pattern side of the trace only. The last hop, which scenarios bind each pattern, stays underivable: cucumber-rs emits no usage report, and joining patterns to scenario steps means compiling Cucumber Expressions, which is checker logic and belongs in a step definition rather than in a command value. Two checks therefore need that join in their steps: the stale-plank join, and the orphaned-step-definition check that reddens a pattern no scenario binds.

Two derived checks report a known weakness, per the Check tooling rule. `plank-inventory` and the `plank-form` rule match a `line_comment` carrying `@planks`, so they see the `///` shape but cannot read what the comment says; rustc's own rule that a doc comment must attach to an item, with clippy run at `-D warnings`, is what makes the placement half of plank form executable. The doubles check is no longer one of these rules, and the reason it could not stay one is a general limit rather than a fault in that rule. `forbidden-doubles` keyed on a type name matching `Mock`, `Fake`, `Stub` or `Dummy`. An ast-grep rule matches the text of a declaration rather than its name node, so a struct merely holding a field of a double's type reddened; and the rule could not read the `@exceptional-double` mark that justifies a real double at all, because an attribute sits between the doc comment and the item, and a rule sees a comment's shape rather than its content. Both distinctions need a reader, so the check left the rule set for a step definition beside the plank joins, which is the same reason those joins live there rather than in a rule.

What the check decides is narrower than the rule it replaced, and deliberately so: every `@exceptional-double` mark names one of the three conditions the Verification agreement permits. A mark that gestures rather than justifies reddens, and so does one that outlived the condition that earned it, which is the failure that actually happens. Its floor is that marks were read at all. It lives in `features/methodology-conformance.feature`.

Enumerating the doubles is the half a run cannot decide, and that is a genuine limit rather than a gap awaiting a better filter. Two selectors were tried and both failed for reasons that generalise. A name-keyed filter matched none of the forty-eight types the tree then declared, because these doubles are named for what they stand in for rather than for being stand-ins. A shape-keyed filter fares no better: the standing example is a real HTTP server on a real port that production reaches exactly as it reaches a provider, so it satisfies no production trait and shares no structural property with what it replaces. A double at a network boundary looks like ordinary code, which is the point of it. So an unmarked double is not caught here and no check here will catch one; the enumeration stays a judgment made at harbour. An empty filter passing every assertion downstream of it is how two earlier versions of this check stayed green while inspecting nothing.

The duplication scanner reports a known weakness of its own: `cargo-dupes` reads functions and not items. Its unit census names closures, methods, functions and trait impl blocks, and it carries no unit kind for a constant at all, so no threshold or flag reaches one. Measured on 2026-07-31 at adoption: the scanner sees the byte-identical `status_line` pair in `src/driver.rs` and `src/flow.rs` as one exact group, and reports nothing for `SELECT_KEY`, `RESPONSE_DEADLINE` and `EXPECT_DEADLINE`, which are duplicated across that same pair of files. The blindness was proven structural rather than a threshold effect by a two-file probe: two byte-identical sources holding only those three constant declarations analyse as zero code units, so lowering `--min-nodes` and `--min-lines` to 1 changes nothing. Read a green from that scenario as covering duplicated functions only. Closing it needs a scanner with an item-level unit kind, or a rule keyed on the constants themselves; a duplicated constant is still found the way these three were, by somebody reading two files.

`cargo-dupes` is a `cargo install` binary, recorded under `## Dependencies` in `RIGGING.md` beside `cargo-llvm-cov` and `ast-grep`. Shipwright installed it and wrote that line in one pass during the 2026-07-31 harbour, which is the route the Rigging read contract specifies: every install and every upgrade belongs to Shipwright at fitting out or at harbour, and recording travels with installing. No dependency routes through Crew, whose charter is the smallest change for one failing target. Noting the route here because it is not visible from the line itself, and a reader who does not know it re-derives the doubt.

A coverage blind spot the summary does not announce: a scenario that drives `tinman` as a child process and then SIGKILLs it loses that child's coverage entirely, because a killed instrumented process never flushes its counters. `src/driver.rs` therefore reads far below its real exercise while the scenarios binding its planks pass: 0.00% on the default tier, which never launches a driver, and about a fifth of its lines on `@sandbox`, which launches one and kills it. `fn main()` likewise reads 0 executions in a run where the driver scenarios are green. Read that shortfall as unattributed, never as unreached: judge reachability from the import and call graph, per the "Current design only" Article. Naming the shortfall as a figure rather than a tier-by-tier claim is deliberate, because the number moves with the scenario mix and a claim of 0.00% everywhere decayed once already.

See `RIGGING.md` for the exact commands. Note the cucumber-rs constraint: `--name` and `--tags` are mutually exclusive, and the exclusion reaches the environment variable too, because `CUCUMBER_FILTER_TAGS` is that same `--tags` argument. A run passing both fails with `the argument '--name <regex>' cannot be used with '--tags <tagexpr>'`, whichever route the tag expression arrives by. So `focused` genuinely cannot carry the tag exclusion the Rigging read contract asks of every verification command, and no rewrite of the value closes it.

What stands in for the exclusion is the anchoring: `--name "^…$"` selects by exact scenario name, so a `@captain` or `@shipwright` skeleton is excluded by carrying a different name. That substitute rests on two properties of the specs, and a scenario name is a regex here, not a literal. A name repeated inside one feature file would run both scenarios, condemned or skeleton included, and a name carrying a regex metacharacter would match the wrong scenario or none. Both properties hold today and nothing enforces either, so they are the subject of a derived `@conformance` check. Tier enumeration sweeps use no `--name`, so they carry the exclusion through `CUCUMBER_FILTER_TAGS` as normal.

## Releases

**Captain owns the release version bump, and it is one coupled edit rather than a field.**

Doctrine gives the bump no owner, so this project closes the gap locally. Shipwright holds the package manifests for dependency work and is a harbour role, while a release is mid-outbound. Crew is dispatched only for a failing target, and a bump has none. Boatswain writes hygiene rather than new content. The gap stays open upstream, so a release that waits for the rule to be worked out again waits every time.

These move together or not at all:

- `version` in `Cargo.toml`
- `version` in `npm/package.json`
- every published schema `$id` URI, across `scantlings/` and `assets/examples/`, which `features/methodology-conformance.feature:every published schema URI names the packaged version` joins to the packaged version and counts
- the git tag `vX.Y.Z`, pushed, because every one of those URIs resolves through it

Two facts about the checks around this coupling, both paid for on 2026-07-29.

`features/methodology-conformance.feature:every published schema URI names the packaged version` compares the URIs against `Cargo.toml`, so it catches a forgotten URI and never a changed contract under an unchanged version. A schema whose content moves while the version stands leaves every URI still naming `@vX.Y.Z`, and the check stays green while that pinned URI serves something the repository no longer contains. The tree was in that state through the 0.2.0 cycle: `sandbox-spec.schema.json` lost a required field while `Cargo.toml` still read 0.2.0. The URI is pinned to a tag rather than to a branch, so publishing the next version is what resolves it, and a consumer reading the pinned URI meanwhile gets the older contract. Read that as the standing shape of the gap rather than as a description of the tree now, which moves every release.

`features/methodology-conformance.feature:both packaged manifests name one version` joins the two manifest versions and reddens when they diverge. Before it nothing joined them at all, so a bump applied to one could reach a registry naming a version that described different contents.

## Outbound

Tinman ships three outbound targets. `RIGGING.md` carries the exact `ship` and `verify` commands for each under `## Outbound`, in the order they run.

**The git tag ships first, and it is a real target rather than bookkeeping.** Every published schema `$id` is `cdn.jsdelivr.net/gh/dmytri/tinman@vX.Y.Z/...`, and jsdelivr resolves that through the tag. An untagged release therefore publishes a full set of URIs that answer 404, and nothing in the tree can tell: the schema-URI conformance scenario reads the repository and finds every URI naming the packaged version, exactly as it should, while every one of them is dead on the network. Measured on 2026-07-31: `@v0.1.2` returned 200, `@v0.2.0` returned 404, `@v0.3.0` returned 200. The 0.2.0 URIs were dead from the day they shipped until 0.3.0 replaced them, so the pinned-URI compatibility story below had never actually worked. The tag's `verify` line fetches a schema over the network for exactly this reason, because it is the only one of the three that a repository-side check cannot stand in for.

The two registry targets release independently of each other.

**crates.io** ships the source crate. `cargo publish` from the repository root is the whole runbook.

**npm** ships `@dk/tinman`, a prebuilt `linux`/`x64` release binary rather than the source tree. Its manifest is durable at `npm/package.json` and everything beside it is staged at ship time, so the staged paths are git-ignored: the `ship` command builds the release binary, installs it at `npm/bin/tinman`, copies `README.md` and `LICENSE` into `npm/`, and publishes that directory. Those four files are the published tarball, because `files` names `bin` and npm adds the manifest, the readme and the licence itself. The package was first published ad hoc from outside the repository at 0.2.0 on 2026-07-28; the manifest here reproduces what that publish shipped.

The release profile sets `strip = true`, and that setting exists for this target. The binary is the package, and the intended invocation is the unversioned `npx @dk/tinman`, so anyone who has not pinned fetches it again on every run and debug symbols are weight none of them can use. Stripping took the binary from 7,221,104 to 5,610,464 bytes, the tarball from 2.7 MB to 2.5 MB, and the unpacked package from 7.2 MB to 5.6 MB. Verification is unaffected, because the scenarios drive the test-profile binary through `CARGO_BIN_EXE_tinman` rather than the release one.

Two versions have to move together: `version` in `Cargo.toml` and `version` in `npm/package.json`. The conformance scenario `features/methodology-conformance.feature:both packaged manifests name one version` joins them and reddens when they diverge. The schema-URI conformance scenario reads the packaged version from `Cargo.toml` alone, so that join is what stands between a bumped crate and a forgotten npm manifest reaching a registry.

**An outbound `verify` line installs and runs the artifact a user would receive. A registry lookup is not a verification.** A lookup answers whether a name resolves, which is a question nobody was asking; it says nothing about what the resolved artifact contains or whether it runs. Both lines here download the published package, install it, and execute the shipped binary, per the Outbound verification policy.

Each line also names the version it expects rather than accepting whichever one answers. The crates.io line reads that version from `Cargo.toml`, so it reddens when the registry is behind the repository.

This is not hypothetical. The previous line was `cargo search tinman`, which exits 0 whatever version comes back. The 0.2.0 release reached npm on 2026-07-28 and never reached crates.io, which stayed at 0.1.2 for the rest of that cycle, and `cargo search` reported success for a day across the gap. The replacement caught it on its first run. Both registries served 0.3.0 on 2026-07-31.

### What the 0.3.0 release found in these lines

The 0.3.0 release was the first to run this rigging end to end, and running it is what found the faults. Three of the four had been sitting in values that looked correct and had never been executed. A `ship` line nobody has run is a claim, and the release is where the claim is tested.

**A directory argument needs a path, not a bare name.** The npm ship line read `npm publish npm --access public`, and npm resolved the bare `npm` as a package name from the registry rather than as the local directory. It answered `403 Forbidden - PUT https://registry.npmjs.org/npm`. The line now reads `npm publish ./npm`. Both forms are visible in a dry run, which is the cheap way to tell them apart: the bare form reports `+ npm@12.0.2` with 1942 files, and the path form reports `+ @dk/tinman@0.3.0` with 4. The 0.2.0 package was published by hand from outside the repository, so this line had never run.

**The npm verify line takes `--prefer-online`.** Without it, `npm install @dk/tinman@latest` failed six times over two minutes with `ETARGET` while `npm view` reported `0.3.0` and `dist-tags.latest` agreed. That is a stale local packument rather than registry propagation, and it makes the line report a good artifact as bad. That failure is the mirror of the `cargo search` one above: one passed everything and one failed everything, and neither answered the question asked.

**A ship or verify line prints its own status.** A release run piped a publish into `tail -12` and read `exit=0` from a publish that had just 403'd, because a pipeline reports the status of its last command. Every line under `## Outbound` now ends by printing its own exit status before propagating it, the way the sweep commands under `## Run data` already do. The printed line survives the pipe even where `$?` does not, so a role that pipes the output still sees the failure.

The general rule under all four: a value in `RIGGING.md` that has never been executed is not yet a value. Prove a changed ship or verify line by running what can be run without publishing, and name in the report what could not be.

## Settled decisions

Decisions a later harbour would otherwise re-derive from repository signals. A finding recorded only in Captain's private notes is invisible to every other role, so it is re-derived every harbour and re-argued every time; this section is where such a decision becomes durable. The watchbill-shape condemnation is recorded under Methodology checks above, in the same spirit.

`pty::launch` and `pty::capture` are a settled keep. Reference analysis shows both have no production caller: every launch path reaches the PTY through `capture_interactive` or `capture_interactive_at`. They were kept at an earlier harbour on the negative-control argument, and the 2026-07-29 harbour re-derived them as unreachable because that decision lived only in Captain's notes. They are not dead code and are not a finding. A harbour that reports them again is repeating a settled decision.

### Tinman stays on serde_yaml, deliberately

The 2026-07-30 harbour reported `serde_yaml` as deprecated and therefore a dependency risk. The survey run against the registry that day reversed the conclusion, so the report and the decision disagree and this record is the decision.

`serde_yaml` was frozen rather than found faulty. The author's own note on the 0.9.34 release says he is publishing no further versions because none of his projects have used YAML for a long time, that he archived the repository, and that an official replacement is not designated. The two releases before it were ordinary bug fixes: 0.9.33 fixed a quadratic parse time, and 0.9.32 fixed compiler warnings. Nothing in that sequence describes a defect a caller inherits. Read `deprecated` in the version string as the author standing down, never as the crate having failed.

That undesignated succession is the whole reason there is nowhere settled to go. Each fork stalled at roughly the moment it was made: `serde_yaml_ng` last published 2024-05-26, and `serde_norway` last published 2024-12-21. `serde_yml` markets itself as the replacement while its own crates.io description opens with `DEPRECATED` and calls itself an unmaintained compatibility shim.

That shim now forwards to `noyalib`, a pure-Rust library advertising zero unsafe code and full serde integration, published as recently as 2026-07-25. Expect a later harbour to find it by following the same chain and to read it as the live answer. Two facts place it: it stood at 0.0.17 on 2026-07-30, and it comes from the author whose previous YAML crate now calls itself unmaintained. Weigh it on its own record when the trigger below fires, rather than adopting it because it is the end of the redirect.

The live lineage is separate and predates the deprecation rather than answering it. `serde_yaml` parsed through `unsafe-libyaml`, a transliteration of the C library. `saphyr` descends from `yaml-rust` through `yaml-rust2`, a pure-Rust YAML 1.2 line, and `yaml-rust2` now states in its own README that it receives basic maintenance and a stable API while `saphyr` takes new features at the cost of a less stable one.

The deciding fact is upstream's own. The saphyr README describes its repository as home to `saphyr-parser`, `saphyr` and "soon-to-be `saphyr-serde`". The official serde integration is unwritten. `serde-saphyr` is a different author's crate at a different repository, filling that gap ahead of upstream, and on 2026-07-30 its newest release was the prerelease 1.0.0-rc.2 against a stable line still at 0.0.29. Adopting it would mean migrating now and migrating again when `saphyr-serde` lands.

So the move is deferred because there is no stable destination, by the upstream project's own account, and the dependency policy pins the version meanwhile.

**The revisit trigger is an event, not a date.** Re-open this when `saphyr-serde` is published, or when an advisory lands that reaches the pinned version. A harbour that re-opens it on the deprecation marker alone is repeating this decision.

One precision the survey bought, because the obvious query gives the wrong answer. `serde_yaml` does carry an advisory, RUSTSEC-2018-0005, so a name-only advisory lookup reports a hit and reads as a live finding. Its affected range is `>=0.6.0-rc1, <0.8.4` and it was fixed in 0.8.4, so it cannot reach a 0.9 pin. Query the advisory databases by name and version together and let the answer decide; on 2026-07-30 the version-scoped query returned nothing.

The environment reaches crates.io directly. A plain `curl` against the crates.io JSON API answered every query in this survey, and one against the OSV API answered the advisory queries. Any note claiming the registry is unreachable from this VM, and that `cargo search` is the only route, describes a past failure rather than a standing property. Prefer the JSON API for a version or a publication date, because `cargo search` reports neither and exits 0 whatever it finds.

### Tinman keeps hudsucker for the proxy

The operator ruled on 2026-08-01, after the field was re-surveyed against the live registry that day. The proxy seam stays on `hudsucker`.

Three real candidates exist. Everything else the search surfaces is a client-side proxy connector, a DNS proxy, or unrelated, so a later survey that returns a longer list has widened the query rather than found new options.

Cost was measured as a marginal figure rather than a standalone one, because a standalone package count answers a question nobody is asking. Each candidate was added to a throwaway crate already carrying Tinman's other fifteen dependencies, and the resolved package count read off that. Measured on 2026-08-01: no proxy crate at all resolves 311 packages, `rama` 363, `http-mitm-proxy` 368, and `hudsucker` 396. So `hudsucker` costs 85 packages where `rama` costs 52 and `http-mitm-proxy` costs 57.

**The weight argument was heard and did not decide it.** The spread is 33 packages in about 400. Against that spread, `hudsucker` states this job as its purpose, where `rama` is a modular service framework in which man-in-the-middle proxying is one capability among many, so adopting `rama` means adopting a framework's opinions on everything else the seam does not need. Two further facts point the same way: `hudsucker` carries 212,852 downloads against `rama`'s 58,965 and `http-mitm-proxy`'s 32,749, and it holds the most recent publish of the three at 2026-07-15, where `http-mitm-proxy` last published 2026-01-24.

One measurement is not comparable to the others, and it is recorded as such rather than repaired quietly. The `rama` figure of 363 was taken at default features, and whether its man-in-the-middle path needs further feature flags was not verified. That figure is therefore a floor rather than a like-for-like reading, and the real gap is likely narrower than 33. A later survey that repeats this comparison should resolve `rama` at the features the seam actually needs before treating the two numbers as comparable.

**The revisit trigger is an event, not a date**, on the `serde_yaml` precedent above. Re-open this when `hudsucker` stops publishing, or when a purpose-built proxying crate appears whose marginal cost is materially lower at the features this seam actually needs. A harbour that re-opens it on the package count alone is repeating this decision.

The specs stay crate-neutral, and deliberately so. Nothing in `features/proxied-egress.feature` names any crate, so a future swap rewrites a handler seam and touches no durable artifact. That is what keeps this a dependency decision rather than a specification one.

### Three pinned versions are held, and approaching-latest is not a reason to move

The operator ruled on 2026-08-01. The 2026-08-01 harbour reported three resolved versions sitting behind current stable, and all three are held: `clap` at 4.6.4 against 4.6.5, `clap_complete` at 4.6.7 against 4.6.8, and `jsonschema` at 0.48.5 against 0.49.2. Every other dependency stood at latest stable that day.

No advisory reaches any of the three, and no spec needs anything a newer version carries.

`jsonschema` is a shipped dependency, carried under `[dependencies]` in `Cargo.toml` with default features off, because `src/expect.rs` validates against a scantling at run time. So the pin reaches what a user receives rather than only what verification builds.

**The `locked` policy exists so that a version moves on a reason rather than on drift.** A version delta is not itself a reason. Read a report of one as an observation, never as a finding, because treating the delta as the trigger makes the policy mean its opposite: it would move every version the moment upstream published, which is the drift the pin was chosen to prevent.

**The revisit trigger is an event, not a delta**, on the `serde_yaml` precedent above. Re-open a pin when an advisory lands that reaches the pinned version, or when a spec or a seam needs something a newer version carries. A harbour that reports these three again on the version delta alone is repeating this decision.

One of the three is not like the other two, and a later harbour should not batch them. `clap` and `clap_complete` are patch moves within a stable major line. `jsonschema` from 0.48 to 0.49 is a minor bump on a 0.x line, where the semantic-versioning contract permits a breaking change, so it owes a real look when its trigger fires rather than a routine bump taken along with the others.

### The default tier's budget stays at 120s, and a red is answered by the suite

The operator ruled on 2026-08-01, on the harbour finding that the default tier was consuming most of its ceiling. Recorded sweeps ran 86576, 86754, 86728 and then 98098 ms, which is 82% of the budget and a rise of about 13%. The rise is explained rather than mysterious: a feature landed nine scenarios.

**The ceiling does not move.** Raising a budget because a run approached it converts the ceiling into a comment, since a limit that yields whenever it binds has never limited anything. The value under `## Tiers` in `RIGGING.md` stays where it is.

The lever, when the tier does redden `features/methodology-conformance.feature:the most recent sweep of each tier is within its declared budget`, is the suite rather than the number. Two moves are available in order. The first is measurement: this project cannot currently read a per-scenario duration, because cucumber-rs emits none, and closing that needs a custom cucumber-rs `Writer`, which is verification support and QM's to write. The second follows from the first, because fixture amortization is only worth spending on the scenarios the `Writer` names.

**Read a budget red as the check working.** It is the signal that the suite has outgrown its ceiling and that the cost is now worth a voyage's attention, which is the whole reason the budget carries a number instead of a note.

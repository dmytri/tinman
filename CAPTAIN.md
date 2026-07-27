> STOP. Captain's notes: non-binding. Captain writes, Captain trims. Anyone else: close this file now.

# Captain Notes

Binding behaviour lives in `.feature` specs and referenced scantlings. History lives in git. These notes carry only what the next cycle needs.

## First action

**The deck is not yours. QM has it.** Voyage 5 is closed and committed at `ee121b8`, one commit ahead of `origin/main` and unpushed. Voyage 6 is authored and unstarted: `watchbill.json` carries 3 watches and 7 targets. The next role is QM in cleared context. Captain's authored artifacts ride uncommitted as work in flight; Boatswain commits them with the voyage's custody.

Run the Opening retrieval anyway and believe it over this paragraph.

## What actually binds Captain. User-confirmed, 2026-07-26.

Two things, and they are absolute. Nothing from conversation reaches QM outside durable artifacts. Captain never writes production code outside the Perturbation policy's named exception.

Everything else is a default, not a wall: write scopes, which role owns which file, and reading anything at all. Captain departs from a default where following it would stall the voyage or leave a fault standing, does the work, and records the reason in one sentence. This is Captain's authority at sea stated plainly, and the user has confirmed it: **do not work as though under-permissioned.**

**Read freely.** Transcripts, source, a dispatched role's progress. The bulkhead is one-directional, and QM's context derives from artifacts Captain already holds, so there is nothing to contaminate by reading. One trap, learned by falling into it: do not infer *doneness* from a transcript or a process table. Those are runtime internals rather than a contract, and the report is the only signal that says finished. A QM run read as stalled, on no live cargo process and a trailing tool result, was in fact substantiating its blockers with `git diff` and `rg` before writing them down, and reported seconds later. Read to narrate, never to decide completion.

**Keep routing writes to the owning role, for the right reason.** Not permission, verification. QM owning the step definitions is what caught an assertion passing on containment, where the erased text was a prefix of the unerased text, and a colour join comparing a 60-row grid against a 24-row rectangle that was false however the program drew. That is worth a round trip. It stops being worth it the moment it costs a cycle to avoid a one-line edit: the `Cargo.toml` `exclude` line and the `plank-presence` rule entry were both that case, and both should have been made without ceremony.

**A recorded departure is one sentence naming what and why.** Not three paragraphs of self-audit.

## FIRST: Tinman executes targets outside the sandbox. Security defect, blocks 0.2.0.

**`record` and `inspect` both launch the target program with the operator's real home, environment, PATH and network.** Established by command, not by reading prose:

- `src/inspect.rs:20` calls `capture_interactive` on a `PreparedProcess` it constructs itself.
- `src/record.rs:195` launches `/bin/sh -c <command>` on a directly constructed `PreparedProcess`. It imports `SandboxSpec` only to write one **into the recorded plan**, so the plan describes a sandbox the recording session never ran in.
- `src/flow.rs:103` and `src/driver.rs:213` are correct: they take a prepared process from elsewhere.
- `grep -rn "PreparedProcess {" src/` names the three construction sites; only `src/bwrap.rs:186` is legitimate.

**`AGENTS.md`'s claim that "tinman record launches its target inside a sandbox" is false today.** It reads as shorthand for a general policy and is in fact a statement about one command that is not true of it. No scenario pins isolation for either command: `features/inspect-command.feature` has three scenarios, none about containment, and `features/sandboxed-launch.feature` does not cover them.

**User ruling, 2026-07-27: Tinman never executes a CLI or TUI outside the sandbox, for any command.**

**The structural fix, and it is the one that cannot rot: only the sandbox backend may construct a `PreparedProcess`.** One invariant catches both bypasses and every future one. The existing reference-boundary branch of `scantlings/proof-contract.schema.json` is per-module, so guarding it that way means remembering to add a contract for each new module, which is the failure mode this project keeps paying for. It wants **a new construction-boundary branch: a named type, and the modules permitted to construct it.** Whole tree, one contract, and a new module that bypasses isolation reddens without anyone having thought to guard it.

**And it needs the negative-control shape.** Asserting a launch ran under bwrap proves plumbing. Proving containment means a target that tries to touch a sentinel outside the sandbox, with the scenario asserting the sentinel does not exist, per the `pty::launch` lesson recorded below.

## Deck state, 2026-07-27

Committed through `8b830b3`, **nine commits ahead of `origin/main` and unpushed**. Voyages 4 through 9 are closed. The assistant now draws a rounded ratatui box capped at 80 columns, writes both halves of each exchange into scrollback above it, carries the session forward with continuous compaction at seventeen whole exchanges inside a 120000-character budget, reports elapsed seconds on a pending call, and cancels on escape. `@logic` 167 scenarios, `@sandbox` 36, both green at custody.

**Safe to play with: `tinman --help`, which executes nothing. Not safe on untrusted input: `record` and `inspect`, per the section above.**

**Verify before trusting any of this.** `git log --oneline -12`, `git status --porcelain`, `git rev-list --left-right --count origin/main...HEAD`, and the three `broad-*` commands from `RIGGING.md`.

**Two roles mischaracterised the rule set this voyage.** Boatswain twice called `scantlings/verification-conformance` a three-rule set covering plank form, quiescence and forbidden doubles, and once concluded from that that no check covers seam-plank presence. `ls scantlings/verification-conformance` reports **four**, and `plank-presence` is what reddened on the unplanked `Exchange::new` earlier in the same voyage. Count the directory rather than trusting a report's prose.

## The assistant. The direction, user-confirmed 2026-07-27.

**The assistant never gains capabilities. Tinman gains subcommands, and the assistant proposes them.** "Help me build a fixture" is not the assistant writing files; it is Tinman growing `tinman fixture new`, proposed and confirmed through `parse_command_line` like every other action. The blast radius of a confused model stays the enumerable set of Tinman subcommands, each with its own scenarios, and every capability remains usable by a human with no model in the loop. `scantlings/assistant-command-boundary.json` keeps enforcing it unchanged.

**One surface, three consumers: a human at a terminal, Tinman's own assistant, and someone else's coding agent.** Nothing is assistant-only, so discoverability is the whole game. Three consequences the user drew out:

- `assets/skill/SKILL.md` is a contract, not documentation. It is what an external agent reads to learn Tinman, so the skill-to-parser check added at voyage 7 keeps an agent from being told about a command that does not exist. It should grow to cover options and the fixture and sandbox vocabulary, not just command names.
- Per-command `--help` is discoverability, not polish. It errored until voyage 6.
- **A machine-readable command surface is missing.** An agent parsing prose help is brittle. Derive it from clap so it cannot drift from the parser, the same trick as the exhaustive match: the compiler owns correctness rather than a document. Undecided whether to write it before the subcommands exist to describe.

**Near:** inspect-and-describe. `tinman inspect` exists, the `COMMAND:` marker fix makes it proposable, and carried command output is already specified. **New:** fixture scaffolding as a subcommand, and multi-step proposals, since one reply is one proposal today and a sequence needs its own confirmation model. That is the real design work, not the file writing.

**The binding constraint is latency, not safety.** At 15 to 110 seconds a turn, a four-step flow is minutes. Good for authoring a test once, bad for anything iterative.

## What the user found by actually running it. Three faults, all Captain's.

1. **It never offers a command.** `src/assistant.rs:16` requires the reply to start with the literal `COMMAND: `, and `assets/help/assistant-instruction.txt` never mentions it. Grep for `PROPOSAL_MARKER` across `assets/` and `features/` returns nothing. The model writes prose, `strip_prefix` fails, every reply falls through to `Response::Answer`, and the whole propose-confirm machinery is built and unreachable. Fix: the instruction states the form, plus a scenario pinning that the form the asset teaches is the form the parser accepts. **This is the two-list problem again, in a third place.**
2. **Markdown fences render raw.** Nothing strips or styles them, and a fenced command also cannot match a prefix, so this compounds fault 1.
3. **The concision brief truncated substance.** `SKILL.md:69-105` covers sandbox blocks, `mounts`, fixture directories and `mode: copy`, and it was all in context. "One line is the target and three is the ceiling" forced a syntax fragment where a procedure was wanted. Captain optimised for the symptom the user named and broke the answer.

**Final instruction copy, user-approved, not yet written to the asset:**

> Answer with the whole answer, at its shortest.
>
> Lead with the TL;DR: the thing that actually answers the question, first, in one line where one line is honestly enough.
>
> When in doubt, say less. The operator can ask for more. Elaboration is theirs to request, not yours to volunteer.
>
> Keep only what the answer cannot stand without. A procedure keeps the steps it fails without; optional refinements go, and so does everything else: no preamble, no restating the question, no summary, no caveats they did not ask for.
>
> Work out what the operator is trying to do. When the goal is unclear, ask one short question instead of guessing.
>
> Show the exact command or plan snippet rather than describing it.

**And a production change that goes with it:** `ask` runs `strip_prefix` on the whole reply, so a reply is either entirely a proposal or entirely prose, and the model cannot say "that does X, here is the command". Change it so the **last line beginning `COMMAND: ` is the proposal and everything above it is the answer**. The boundary is unchanged: anything after the marker still goes through `parse_command_line` and nothing else.

## Assistant work authored in conversation, not yet in specs

Every item below is user-confirmed and owes scenarios.

- **Bare `tinman` runs the assistant** when all four hold: no subcommand, no arguments, stdin not redirected, and an interactive terminal. Any one missing falls back to conventional help. It shows the name and tagline and the box, no Commands block. The stdin condition is the one that matters: `tinman < plan.yaml` must not open a UI that eats the input.
- **Leaving clears the screen, option C.** Alternate screen during the session, and the transcript written back into scrollback on the way out. Rejected A, a clean exit that loses the transcript, because the usual reason to leave is to run the command just given. This supersedes the inline-viewport rationale in `features/interactive-help.feature`, which must be reworded when the change lands.
- **The question is marked by background colour, not foreground.** Full transcript width, so it reads as a block rather than a highlighter smear. **It needs a non-colour fallback**, a leading marker, because a background block under `NO_COLOR` degrades to no structure at all, which is worse than the foreground version degrading to nothing missing.
- **Markdown rendering: `tui-markdown`, `default-features = false`.** Repository `github.com/joshka/tui-markdown`, a ratatui maintainer, so it tracks the framework across its 0.x breaks. Converts straight to a ratatui `Text`, which is the type `insert_before` wants. Its default `highlight-code` feature pulls `syntect` and `ansi-to-tui`; leave it off, and highlighting later is a flag rather than a rewrite. Caveat: 0.3.x, and release cadence was not checked. Not yet recorded under `## Dependencies`.
- **Scantlings go into the assistant context, features do not.** Measured: `features/` 94794 chars, all scantlings 37960, the six user-facing ones 22087, `SKILL.md` 6025. The user-facing subset is `harness-plan`, `sandbox-spec`, `tom`, `driver-protocol`, `interaction-log`, `skill`. The argument that decides it: **a scantling cannot drift, because a `@contract` scenario pins it**, while the skill can and did. Features are 24k tokens of implementer-facing material every turn, half of it internal method.

## Markdown testing, and the mdbook decision

**Tier 1, parse-check, nearly free.** Every `tinman ...` command line in a fenced block in any shipped markdown is fed to `parse_command_line` and must be accepted. Runs nothing, no fixtures, no side effects. **It subsumes the bespoke skill-to-parser check** into one rule over `README.md`, `SKILL.md` and `AGENTS.md`.

**Tier 2, execute.** Fenced blocks tagged as plans get run. Needs the fixture subcommand first, since doc examples elide and need fixtures to be honest. Tag with CommonMark **info strings**, the standard mechanism, rather than inventing a comment marker. Precedent is Rust doctests and `mdbook test`.

**This is a product feature, not internal plumbing.** "Test the examples in your own README" is a real capability for a testing tool, on the same discoverable subcommand surface as everything else.

**mdbook: no, and the testing rationale specifically fails.** `mdbook test` compiles Rust doctests; it cannot run a Tinman plan or check a command line, so the extractor is ours to write either way. What it would buy is a website, against a 207-line total doc surface, plus a second outbound target with its own verification obligation and a fourth surface to drift on. Revisit as a marketing call when there is prose to warrant a book, not as a testing one.

## inspect, and the discoverability surface. User-confirmed 2026-07-27.

**`inspect` must work for plain CLIs, not only TUIs**, per the standing preference that Tinman stays useful for plain command-line testing.

- **A program that exits on its own is read as a stream**, its complete output, unbounded by terminal height. **A program still running is read as a screen.** Exiting is observable, so the rule needs no flag and no guessing. Reading a plain CLI as a screen would let output longer than the terminal scroll off, making assertions depend on terminal height, which is the same quiet-wrong-answer class as the `cwd` bug.
- **Undecided, Captain owes the user an answer:** roles for stream output. Either lines become `listitem`s in a `list`, good for `ls`-shaped output and wrong for prose, or blank-line blocks become `article`s in a `log`, reusing the rule that already exists and coarse for line-per-item.
- **`--help` is ground truth**, from the binary being tested, at the installed version, inside the sandbox. **`man` opportunistically**, absent from a minimal bwrap sandbox because we do not bind `/usr/share/man`, and widening the sandbox to read documentation is not worth it. **tldr, including `tlrc`, as hints only**, never as the basis for an assertion: a tldr page describes some version of a program the way our help text described a parser it had drifted from.
- **Captain was wrong about the network objection to tldr, and the user caught it.** Network is denied to the *target*, not to Tinman: the inference call is already made by Tinman outside the sandbox. A tldr fetch would be the same shape, same tier, capture-time only. What survives is the drift argument alone.

**Discoverability: one surface, three consumers, everything derived from one clap definition and one prose file.**

- **`tinman man`** emits roff via `clap_mangen` 0.3.0, and **`tinman completions <shell>`** emits a script via `clap_complete` 4.6.7, both **at runtime rather than committed**. Nothing generated sits in the tree, so nothing can drift and no conformance check is owed to keep it current. It also works after `cargo install`, which a shipped man page does not, and packagers pipe the output into their build.
- **The named-commands floor moves to seven**: `record`, `test`, `inspect`, `driver`, `help`, `man`, `completions`. That floor existing is what makes adding commands safe.
- **`SKILL.md` stays the one prose source**, read directly by agents and rendered for humans by `tinman help <topic>` through the `tui-markdown` renderer the assistant already needs. **`mandown` was declined**: rendering the skill as roff puts it on the drift-prone side, where today it is checked against the parser.
- **The skill and the man page are not one document.** A man page is exhaustive reference, a skill is teaching material. What unifies is the sources: the CLI surface from clap, the concepts from prose.
- **A clap-derived machine-readable manifest is still missing**, and it is what an external coding agent should read instead of parsing prose help. Undecided whether to write it before the subcommands exist to describe.

## Scheduled work

Every item names its pivot and its role. No "someday".

**Voyage 5 items, all authored 2026-07-26 and now live for QM.** Kept here as the diagnosis behind each target, which the watchbill cannot carry. Every item was earned by a voyage-4 or 4a report. Item 1 is a shipped defect a user can hit today, item 2 is the architecture divergence two voyages failed to close, and 3 through 6 are checks that would have caught the rest without a human reading anything. Item 6 gained a ninth target from the user's ruling: see the session decision below.

1. **A stalled inference provider hangs Tinman indefinitely. This is a shipped defect, not a harness one.** `src/inference.rs:276` calls `ureq::post(&url)` with no agent and no configured timeout, so a provider that accepts the connection and then stops answering never returns. `tinman --help` sits on that path. Established by QM's sweep consuming the entire 600s foreground budget inside the call, and corroborated by a prior 347894ms `@inference` weather line against a ~64s norm. The existing scenario covers an **unreachable** provider, which fails fast; nothing pins a **stalled** one. QM's 120s bound is in verification support and protects only our suite. Write the scenario, and note it needs a server that accepts and then withholds, which is a real local listener rather than a double.
2. **The `cwd` divergence is still open. Voyage 4 did not close it.** Confirmed by command: `src/pty.rs:111-115` shows `capture_interactive_in` **still taking `cwd: Option<&Path>`** as a third parameter, and `src/process.rs:14` carries `PreparedProcess.cwd` set `None` at all three construction sites, `inspect.rs:24`, `record.rs:200` and `bwrap.rs:190`, and **read nowhere in `src/`**. `src/flow.rs:68` threads `run.cwd.as_deref()` down the sidecar, so the flow scenario is green on the sidecar path and the `@contract` scenario is green because the field serializes.

   **The route is already built, and this is the part worth reading before authoring.** `scantlings/pty-sandbox-boundary.json` already governs `src/pty.rs`, and its own rationale already states the invariant being violated: *"The PTY runner takes a `PreparedProcess` and launches it verbatim."* Its `forbiddenReferences` list keys on bwrap symbol names, so a sidecar parameter walks past it. Add `"requiredReferences": ["prepared.cwd"]` to that contract: the key is optional in the meta-schema, already exercised by `assistant-command-boundary.json` for `parse_command_line`, so the checker implements it and **no meta-schema change and no new machinery is needed**. It reddens today. Pair it with the sidecar parameter in `forbiddenReferences`, because `requiredReferences` alone lets Crew write `cwd.or(prepared.cwd)` and stay divergent. Leave `bwrap.rs`'s own `--chdir` alone: building argv is a backend's job and the argument-vector policy contract governs it.
3. **Nothing reddens a production seam carrying no plank at all.** The rule set is exactly three files, `forbidden-doubles.yml`, `perturbation-quiescence.yml` and `plank-form.yml`, and all three check the *form* of a plank that exists. Article 9 is the one Article with no executable check, so a new unplanked seam passes every gate. Both QM and Boatswain named this independently, and Boatswain had to verify the touched seams by **reading** `src/process.rs`, `src/inspect.rs` and `src/record.rs`. A rule keyed on a `pub` declaration lacking a `@planks` docblock closes it.
4. **No derived check reddens provisional-plank drift either.** QM found all five spent `@planks-provisional` annotations by hand. Both plank joins in `features/methodology-conformance.feature` cover `@planks` alone. Same rule set, same pass as item 3.
5. **`the driver exits when its stdin closes` asserts nothing in its second step.** `And the driver leaves no session sandbox directory standing` cannot fail: the only session-directory creation is `src/driver.rs:179-180` inside `launch`, and that scenario's Given launches nothing. The step definition is honest; the precondition starves it. Strengthening the Given to run a session gives it teeth and moves the scenario to `@sandbox`, since its assertion becomes isolation itself.
6. **`scantlings/driver-protocol.schema.json` declares no `required` list on `params`.** Its own description reads "Every method except launch addresses an existing session", and both new invalid-params scenarios send captures carrying no `session` and validate anyway. The missing-parameter scenario omits two parameters while naming one. Prose that binds nothing is the same fault as a property outside `required`. Owe a per-method `required` list.
7. **Terminal echo is unpinned.** `src/driver.rs` now prepends `const SILENCE_ECHO: &str = "stty -echo"` to every launched session. Boatswain judged it inside `launch`'s planked steps and flagged the judgment as **read, not run**. The real behaviour worth pinning is that a program's own output is distinguishable from the driver's keystrokes, which is what the flake violated.
8. **A derived budget check is now possible and absent.** `target/tinman-weather.jsonl` carries per-tier wall clock and nothing reads it. The Verification agreement's budget check needs no new instrumentation. Setting the ceilings is a user decision; the observed floors are in "Settled: no budgets" below.

**Next harbour, Shipwright.** Carried forward, plus one new item.

- **Possible behaviour-stale plank.** `src/driver.rs` `launch` gained `@planks("the failure reports the selection did not reach the {string} named {string}")`. It joins a current pattern so it is neither stale nor malformed and was correctly not a custody foul, but `launch` is the session-start seam and that step asserts an activation failure. The pattern join cannot see this by construction. Harbour's coverage triage is the net.
- **Closed, do not re-derive.** The two `src/inference.rs` planks that named orphaned patterns were rewritten this voyage and the orphan check is green. The nine orphaned step definitions harbour flagged are gone.

- **`src/driver.rs` reads 0.00% in every tier** while its 24 binding scenarios pass. `tests/cucumber/support.rs:952` SIGKILLs the child, and a killed instrumented process never flushes counters. The whole out-of-process surface is unattributed, so no coverage claim about the driver means anything until this is closed.
- **`forbidden-doubles` keys on the type names `Mock|Fake|Stub|Dummy`.** `LocalProvider` is a real double, correctly marked, that the rule would not have caught unmarked. Any double named otherwise is invisible.
- **Derive a schema-property-against-production-field check.** A scantling only validates the shape production emits, so a declared property production never emits, or emits and never reads, is untested in both directions. Voyage 5 item 2 pins the one live instance by hand; this is the general check. See the corollary under the false-green section.
- **Decide `ratatui-testlib`'s harness layer.** Leaning no on architecture: launch must happen inside bwrap and the PTY runner accepts only a prepared process, while `TuiTestHarness` owns spawning and ships no sandbox. Six weeks at v0.1.0 with one maintainer likely fails the well-maintained bar for a foundational layer.

**Next release, whoever ships.** `sandbox-spec.schema.json` lost the `fixture` values from `home` and `env.*.from` at the 2026-07-25 harbour. That is a breaking change to a published schema, so the release carrying it owes a version bump. The bump still has no owner in doctrine; see the gaps below.

**Held, not scheduled.** `jsonschema` 0.48.5 and `generic-array` 0.14.7 sit behind latest with 0 semver updates available. The `locked` policy holds them until a spec or a Captain decision moves them. No spec needs either. Revisit at next harbour.

**Settled: no budgets.** `budget: 120s` is struck from `## Tiers` by user decision. It was a ceiling no command produced and no check read, and `@inference` alone had exceeded it. Better no budget than an unenforced one. Observed floors if one is ever wanted: `@logic` ~24s, `@sandbox` ~26-34s, `@inference` ~63s with one 348s outlier during the flake. The producer question is now answered: every tier sweep appends its own wall clock to `target/tinman-weather.jsonl`, so a derived check has something to read. The `coverage` commands still deliberately skip the append, so harbour's own regression records nothing, and a budget check reads sweeps only. Reintroducing ceilings is a user decision; see voyage 5 item 8.

## The false-green pattern. The most expensive thing this project has learned.

Fifteen instances now, every one green while asserting nothing. The six older ones: a perturbation discharged by deleting its line; a `menu` role the deterministic pass already produced; a launch answering `ok:true` before the program started; a recorded plan carrying no expectation; two activation scenarios asserting text the fixture drew anyway; and a scantling that is valid JSON and an invalid schema.

Two more this harbour, both caught only by planting:

- **The gplint glob.** `features/**/*.feature` matches zero files in gplint's glob, because `**` must consume at least one directory segment and `features/` is flat. It exits 0 having linted nothing. Captain and Shipwright hit this independently, an hour apart, both reaching for the idiomatic form. The working values are `features/**` and `features/*.feature`. `RIGGING.md` carries the former and `AGENTS.md` records the trap.
- **A plank string shared across six seams.** `@planks("a process is prepared and launched")` is carried by `process.rs`, `sandbox.rs` three times, `bwrap.rs` and `pty.rs`. The string join is green no matter which of the six is dead, so a shared plank string makes the join structurally unable to see a dead seam. The trap is real; the seam it was found on turned out not to be dead. See the negative-control rule below.

Two more on 2026-07-26, both `cwd`, and the second is the worst class yet because no scenario covered it at all:

- **An optional schema property is an attestation that cannot fail.** `prepared-process.schema.json` declared `cwd` outside `required`, so a `PreparedProcess` that has never carried the field serializes without it and validates. The `@contract` scenario passed for two full cycles while the struct and the schema disagreed. Any optional property in a scantling is a promise nothing checks: read the `required` list, not the property list, when asking whether an attestation binds.
- **A schema can promise a capability production silently discards.** `harness-plan.schema.json:51` declares `cwd` on a `run:` step, and `additionalProperties: false` there means the key is explicitly *permitted*, not tolerated. `RunForm::Full` in `src/plan.rs:57-63` has `command`, `status` and `stdin` and no `cwd`, and the enum is `untagged` with no `deny_unknown_fields`, so serde drops the key without a word. A plan writing `cwd: subdir` validates against the published schema and then runs in the wrong directory. No error, no warning. This is quieter and therefore worse than the `home: fixture` case, which at least failed loudly at deserialize. Grep found it in one command: no reader of `cwd` existed anywhere in `src/` outside `pty.rs`.

Two more from voyage 4's own reports, and the first is the one to study, because a lever designed against this exact pattern failed:

- **A `required` property proves a field exists. Nothing proves it is read.** The note above congratulated itself on moving `cwd` into `required` so that Crew could not thread another sidecar and skip `PreparedProcess`. Crew added the field, left it `None` at every construction site, read it nowhere, and threaded the sidecar anyway. Both scenarios went green honestly. Tightening a scantling raises the floor by exactly one inch: the field's presence and type. The capability's *wiring* is a different claim and needs a different check.
- **A precondition that seeds nothing makes its assertion vacuous.** `the driver exits when its stdin closes` asserts no session sandbox directory is left standing, in a scenario that launches no session. Four green steps, one of them asserting against a state that could not exist. This is the Given-side twin of the older cases, which were all assertion-side: the step definition is correct and the setup starves it.

Thirteenth, found 2026-07-26 while answering a plain user question, and the first one that is a production command rather than a scenario:

- **`tinman replay` is declared and does nothing.** `src/cli.rs:35` declares `Replay` in the clap enum, so it appears in `tinman --help`, and `accepted_commands()` derives from clap, so the interactive assistant proposes it and `parse_command_line` accepts it. `src/main.rs` has no arm for it and ends the match at line 72 with `_ => {}`, which swallows it: the command exits 0 having done nothing. All six `replay.feature` scenarios are green because they drive the library seam and never the subcommand. The wildcard arm is what makes it invisible; an exhaustive match would not compile. It also takes no plan argument, so the intent is under-specified as well as unwired, and it lands on the headline capability: capture may infer, replay is deterministic.

Fifteenth, 2026-07-26, and Captain wrote it into a scenario Captain never opened:

- **An absence assertion disarmed by a change to the asset it names.** `features/inference-availability.feature:degraded help omits the assistant prompt` asserted the help output does not contain the body of `assets/help/assistant-prompt.txt`, as a contiguous string. Captain grew that asset from one line to two and drew it in a ratatui box, where line one becomes a border title and line two a bordered row. The contiguous two-line body can therefore never appear, whether the assistant opened or not, so the scenario passes with the `expansion.is_some()` gate in `src/main.rs` present **or removed**. Boatswain established the mechanism from the diff and labelled the conclusion unverified, wanting the planted red QM owes at adoption. Rewritten to assert against the terminal object model, the same place its positive twin already asserts.

**The lesson is the standing corollary, paid for a second time in one session:** a change to a definition narrows what every scenario counting on it may claim, so re-read them in the same pass. Captain re-read the scenarios that assert the box's **presence** and never the one that asserts its **absence**. Absence assertions are the ones that go quiet, because nothing about their output changes when they stop testing.

Seventeenth, found by Boatswain 2026-07-27, and this one is in the tooling rather than a spec:

- **A focused run against a scenario name that no longer exists selects zero scenarios and exits green.** `the implementation carries no standing perturbation` was folded into `the verification-conformance rule set reports no match`. Any role reaching for the old name by memory gets a clean bill from a run that checked nothing. Same shape as the gplint glob reading zero files. `ast-grep scan` is the command that actually answers quiescence. **A focused run reporting `0 scenarios` is a failed selection, never a pass.**

**The test: when a scenario goes green early and cheaply, ask what it would take to make it red. If nothing would, the scenario is the defect.**

**Corollary from the thirteenth: a wildcard match arm is the production form of a glob that reads nothing.** Both report success for work never done, and neither can be seen by a green run. Prefer an exhaustive match wherever the compiler can enforce it, so an unhandled case is a build failure rather than a silent exit 0.

**Corollary, now twice earned and sharper than before: a scantling only ever tests the shape production emits.** A property production never emits is untested. A property production emits but never *reads* is also untested, and that one survives a `required` list. Both `cwd` instances were found by hand, prompted only by an old note. The join that would catch either is schema property against production field, plus a reachability question the schema cannot ask: does anything consume it? Nothing we own runs that. Worth a derived check, and item 1 of voyage 5 is the manual instance of it.

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
2. **An adoption proof leaves no durable trace.** A green looks identical proven or not. Five proofs now: the proof-contract meta-schema and the gplint command at the 2026-07-25 harbour, and three checks QM planted red on voyage 4. Every one of them was proven in a session that has since been cleared, so the tree carries no record that any of them can fail. A proof only its author can see is one clear from being unproven again. The run record has a slot-shaped hole here: it records that a target passed, never that it was ever observed to fail.

## Design decisions that bind (user-confirmed)

- **Every driver call except `launch` names its session, and an omitted session is an error rather than a default.** User ruling, 2026-07-26, on the reading that a driver holds several sessions at once so defaulting is a guess. The schema's prose had promised this while its `params` object declared no `required` list, and two green scenarios sent captures carrying no session. The constraint now lives in the `request` definition's conditional branches, because `params` alone cannot see which method it belongs to, and declaring a property there permits it rather than requiring it. The two capture scenarios gained the session, and a third scenario asserts the omission is rejected. Concrete wire messages in those scenarios carry a `{session}` token the step replaces, since the driver assigns the identifier at run time and no literal can be written into a spec.
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

`intent/idea.md`, `intent/isolation.md` and `intent/help.md` are historical intent sources, each carrying a banner saying so, and the whole `intent` directory is in the crate's `exclude` list. Do not treat them as requirements and do not report their staleness: `intent/isolation.md` shows `from: fixture` which production never had, and `intent/help.md` shows a Commands block listing `replay` and omitting `driver`. Both are expected. Two scantling descriptions cite them as the source of an invariant, `assistant-command-boundary.json` and `pty-sandbox-boundary.json`, which is provenance rather than a live reference: the invariant's current statement is the scantling's own, and the scenario that discharges it.

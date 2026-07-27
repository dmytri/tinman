> STOP. Captain's notes: non-binding. Captain writes, Captain trims. Anyone else: close this file now.

# Captain Notes

Binding behaviour lives in `.feature` specs and referenced scantlings. History lives in git. These notes carry only what the next cycle needs.

## First action

**Voyage 10 is sailing. QM was dispatched 2026-07-27 at base commit `2af1a5c`.** `watchbill.json` carries 14 watches and 65 targets, sailing whole by user decision rather than split.

**QM returned 2026-07-27 with the watchbill unspent: 3 of 13 watches green, watch4 blocked, 9 untouched.** Watches 1, 2 and 11 are spent and green by fresh focused runs. That green is uncommitted.

**The prediction held: QM held the 13-watch copy it read at open.** It reported against 13 and never saw watch14 or watch15. Both stand for the next QM, which resumes at watch3. Verified by extraction at write time: all 68 references resolve to real scenario names, `npx --no-install gplint "features/**"` exits 0.

**The blocker is a dependency, and Captain verified it rather than taking QM's word.** `clap_mangen` and `clap_complete` are recorded in `RIGGING.md` at lines 67-68 and appear in neither `Cargo.toml` nor `Cargo.lock`. Installation is fitting out and routes to Shipwright, never Crew, per the Rigging read contract. Four watch4 targets redispatch once it lands, seam cluster `src/cli.rs` plus the dispatch in `src/main.rs`.

**QM worked watch11 out of watchbill order, and it was right to.** A target in watch1 and one in watch2 could not go green until watch11's stream reading existed, so a mate working from watch1's evidence alone would have invented a reading watch11's own scenarios then contest. Six targets batched onto one seam. Verification disagreed with the watchbill order and verification won, which is the Watchbill policy working as written.

Run the Opening retrieval first and believe it over this paragraph. Last verified 2026-07-27: HEAD `8cc9c2c`, `git rev-list --left-right --count origin/main...HEAD` reported `0 0`, and the tree carried 21 uncommitted Captain artifacts as work in flight rather than dirt, so they ride to QM uncommitted and Boatswain commits them with the production change they order. Expect `watchbill.json` present, no `@captain` or `@shipwright` scenarios, and no live `PERTURBATION`.

**Captain runs in the main loop. Operator preference, stated 2026-07-27.** Captain is the only human-facing role and outbound may run only in the human-facing session, so Captain keeps the operator's seat and the internal roles are dispatched away from it. Dispatch QM as a context-isolated subagent rather than clearing this session and becoming QM: that satisfies the bulkhead, keeps Captain on deck, and is the route doctrine prefers wherever the runtime supports it. **Captain never assumes QM.** Dispatch thin, per the contract: role and base commit, nothing else. The watchbill at its home is the channel and the durable artifacts are the hand-off.

**What Captain owes while QM is away.** Dispatching into isolation hides the work, so narrate it: plain prose, at least every couple of minutes, on what is running, what it has produced and what it is waiting on. Read QM's progress to narrate, never to decide whether it has finished. The report is the only signal that says finished, and a QM reading as stalled on a quiet tool result has twice been caught mid-`git diff` and reported seconds later.

**Then, in order.** When QM reports the watchbill spent and its targets green, dispatch Boatswain for post-implementation custody, carrying the job, the base commit and QM's advanced target references. When Boatswain reports passing verification, a clean tree and a local commit, summarize and offer outbound. Do not order a full regression: it is a harbour action and harbour is its only trigger, so offer harbour where a whole sweep is wanted. The crate ships with `cargo publish` and verifies with `cargo search tinman`, both from `## Outbound` in `RIGGING.md`.

**Expect these to be the expensive watches.** Watches 10 through 13 specify a capability that exists in no form yet: reading documentation, probing it in the sandbox, and writing the result as a plan. A blocker from those is expected rather than surprising, and the user has already chosen to sail them in this voyage rather than split.

## What actually binds Captain. User-confirmed, 2026-07-26.

Two things, and they are absolute. Nothing from conversation reaches QM outside durable artifacts. Captain never writes production code outside the Perturbation policy's named exception.

Everything else is a default, not a wall: write scopes, which role owns which file, and reading anything at all. Captain departs from a default where following it would stall the voyage or leave a fault standing, does the work, and records the reason in one sentence. **Do not work as though under-permissioned.**

**Read freely.** Transcripts, source, a dispatched role's progress. One trap, learned by falling into it: do not infer *doneness* from a transcript or a process table. The report is the only signal that says finished. Read to narrate, never to decide completion.

**Keep routing writes to the owning role, for the right reason.** Not permission, verification. QM owning the step definitions is what caught an assertion passing on containment and a colour join comparing a 60-row grid against a 24-row rectangle. That is worth a round trip. It stops being worth it the moment it costs a cycle to avoid a one-line edit.

**Do not evade doctrine on `Rule:` usage.** User correction, 2026-07-27. A `Rule:` carries durable context only: the why, the trap, the history. The requirement lives in the scenario and nowhere else. Writing a `Rule:` as a normative headline with rationale attached is the evasion, and the corpus has old examples of it that are not licence. Every `Rule:` authored on voyage 10 was rewritten once for exactly this.

## Voyage 10, authored 2026-07-27. What it carries and why.

**The security defect is the reason it exists.** `record` and `inspect` both launched their target with the operator's real home, environment, PATH and network. Confirmed by command: `grep -rn "PreparedProcess {" src/` returns `src/record.rs:196`, `src/inspect.rs:20` and `src/bwrap.rs:186`, and only the last is legitimate. `src/backend.rs:40` already carries `resolve(requested, allow_unsafe)`; neither command calls it, so the machinery was built and never wired. User ruling: **Tinman never executes a CLI or TUI outside the sandbox, for any command.**

The fix is structural: `scantlings/prepared-process-construction-boundary.json`, a new **construction-boundary branch** in the proof-contract meta-schema, bounding construction of one named type over a whole search tree. A per-module reference contract covers the modules someone remembered; the modules that bypassed the backend were exactly the ones nobody had.

**Consequence to expect:** every `record` and `inspect` scenario that launches now needs bwrap, so they moved to `@sandbox`. That tier gains roughly ten scenarios. Observed floor was 26 to 34s at 36 scenarios against a 120s budget, so headroom is fine, but the budget check is live and will say so if not.

**The TOM grew a stylesheet.** User question, and it reframed the voyage: if this were a webapp we would have HTML and CSS in assets, so what is the Tinman equivalent? Answer: the TOM tree is the HTML and it already existed; the CSS is computed style per node, which the model was throwing away. `alacritty_terminal` hands Tinman an attributed cell grid, the builder reads those attributes to derive roles, and then discarded them. So `tom.schema.json` gained `style` per region and `cursor` on the model, and `assistant-ui.schema.json` stopped confessing that colour and cursor are properties the model cannot carry. It is also a product capability that was missing outright: a test author could not assert *the error line is red*.

**Style is absent where a region's cells disagree.** That is deliberate: one computed style for a region drawn two ways is a summary, and asserting against a summary is asserting against Tinman. Absence means mixed, not plain; plain reports the defaults explicitly, so the shape is uniform.

**The assistant.** Alternate screen for the session with the transcript written into scrollback on exit; one 72-column measure shared by the box and the transcript; the question marked by a background block running the full measure, with a leading `> ` marker as the NO_COLOR fallback; markdown rendered through `tui-markdown`; the name and tagline coloured apart from the help body; bare `tinman` opens the assistant on four conditions. The `COMMAND: ` marker is now taught by the instruction asset and joined to the parser by a scenario, and the last marked line is the proposal with the prose above it the answer.

**Discoverability, per the standing direction that there is one surface and three consumers.** `tinman man` and `tinman completions <shell>` emit at runtime rather than shipping generated files. The seven-command floor is named. The skill-to-parser check widened into one sweep over every shipped markdown file, and three new joins stop the skill drifting again.

**The skill was lying in two places and nothing reddened.** It named nine roles the model has never produced, including `table`, `dialog`, `statusbar`, `message-pane` and `treeitem`, and its driver examples used an `op` key and a top-level `session` where the protocol has been JSON-RPC 2.0 with `method` and `params` for two voyages. Both fixed, both now checked. Verified after the fix: every fenced example in `SKILL.md` validates against the scantling it illustrates, and no role token it names is outside the model's enum.

## Watch14, authored 2026-07-27 from the tldr-pages research

Read [the style guide](https://github.com/tldr-pages/tldr/blob/main/contributing-guides/style-guide.md) and [CONTRIBUTING](https://github.com/tldr-pages/tldr/blob/main/CONTRIBUTING.md) before touching any of this again. Both were fetched, not recalled.

**A tldr example is a template, and watch12 assumed it was a command line.** The syntax is fixed by the style guide: `{{path/to/file}}`, `{{[-o|--output]}}` to let a client choose short or long form, `{{option1|option2}}`, `{{1..5}}`. Every watch12 scenario used a placeholder-free example, so nothing said what happens to `cp {{path/to/source}} {{path/to/target}}`. Run as written it is refused, and inspection then reports an unhonoured claim it manufactured itself. **User ruling: substitute what Tinman can safely fill, report the rest as untestable as written.** A path placeholder becomes a real fixture inside the sandbox; the plan records the line Tinman ran and the example it came from as two separate facts, so the page is never credited with words Tinman assembled.

**Credit was covered for naming and missing for examples.** Watch10 credits the project when a page informed naming; nothing credited a page that supplied an example. Two scenarios close it, mirroring watch10's shape, floor control included.

**mandoc is installed and recorded.** `man emits a roff man page` asserted a title macro and the command names, which malformed roff satisfies. `mandoc` is the language's own parser and the independent tool the standing preference asks for, so the `@contract` scenario runs it. Installed by apt on this VM under the Blocker policy's environment-and-tooling row, recorded under `## Dependencies`.

**The severity threshold is the whole trap, and the scenario says "error" for that reason.** Measured on this VM, true exit codes read without a pipe: a well-formed page exits **2** under a bare `mandoc -T lint`, because a missing date and a lower-case title are a WARNING and a STYLE. Reading that as failure would redden a page that is fine. Under `-W error` the same page exits **0** and a page with an unknown macro exits **3**. So `-W error` is the threshold, the check is falsifiable, and it does not fire on cosmetics. Measuring it also caught the pipeline-exit trap live: `mandoc ... | head; echo $?` reports `head`'s status and reads 0 for a page mandoc rejected. Use `PIPESTATUS` or no pipe.

## Watch15, authored the same day, and a correction worth keeping

**The keypress notation is watch15.** tldr pages carry `<Ctrl c>`, `<Esc><u>`, `<ArrowUp>` in a documented syntax, so a page for a full-screen program documents the interactions and not only the invocation. That is the half of a TUI's documentation no shell can test, and it is a plan step in a notation somebody else already standardized. Two scenarios: a recognised keypress becomes a key step that replays, and an unrecognised one is reported untestable as written with the plan-not-empty floor. Sending it as text would type literal angle brackets into the program, which is the placeholder fault wearing a different hat.

**The house-style pass landed on `assets/help/tinman.txt`.** Checked against their guide rather than assumed: the command descriptions were already imperative, already unstyled, already using `stdin`/`stdout`, and already ordered with `help` last. The one real gap was that the help text carried **no examples at all**. Four now, simplest first and progressively more complex, with a scenario passing every one through the parser. That direction was genuinely unguarded: the Commands block is checked both ways, and an example line escapes both checks because it carries a subcommand, flags and arguments together in the one form the parser actually receives. `SKILL.md` and the man page are not in this pass: the man page is generated from clap doc comments, which is production code and not Captain's, and `SKILL.md` is bound by three bundled-skill scenarios that make it a bigger and separate job.

**Correction, and the reason to keep it.** Captain recommended holding both on the grounds that rewriting the help asset would redden a scenario mid-flight. That was wrong and one grep settled it: `src/help.rs:10` inlines the asset with `include_str!`, so a run in flight uses the compiled binary and an edit lands at the next rebuild. The real constraint is narrow, keep the Commands block, the Options block and the single `{{tagline}}` intact, and it is checkable. Verified after the edit: 1 tagline, 7 commands, 2 option lines, 4 example lines all parsing as Tinman commands. **The recommendation to hold was Captain managing its own uncertainty rather than a real risk, and the user was right to push on it.**

**The README install idiom is prose, not a fenced block, and that was deliberate.** `every Tinman command line in shipped markdown is accepted by the parser` sweeps README fences, and `tinman man > /usr/local/share/man/man1/tinman.1 && mandb` is a shell line rather than a Tinman command line. Prose keeps it out of the sweep. If a later voyage wants it checked, the sweep's extractor is what needs the scenario, not the README.

## Standing preferences the user has stated

Read these before authoring any spec.

- **Whenever the consumer is a human at a terminal, it should be as beautiful as the land of Oz.** An exemplary modern TUI, to the highest usability and accessibility standards, and always sandboxed. User-confirmed 2026-07-27. Tinman reads terminal programs for structure, naming and presentation, so a Tinman interface without them argues against the product. This is falsifiable rather than a matter of taste, and the way to keep it falsifiable is the TOM: Tinman's own screen is checked by the instrument Tinman sells.
- **The accessibility floor that a terminal program can actually break is meaning carried in colour alone.** Contrast belongs to the theme and wording to the copy. Every distinction drawn in colour needs a second carrier, and NO_COLOR is the test.
- **Styling reaches a terminal and no other stream.** A colour escape in a pipe is corruption of somebody's data.
- **Prefer a concise attestation on a scantling over verbose behavioural scenarios**, and audit freshly authored specs against that, not only inherited ones.
- **A scantling based on a well-known standard beats a bespoke one**, and **prefer specifications over implementations**. Applied: JSON-RPC 2.0, WAI-ARIA, JSON Schema 2020-12, CommonMark info strings.
- **Prefer an independent tool over a bespoke checker**, and never let two checkers hold one rule.
- **`Rule:` prose carries durable context only.** Requirements belong in scenarios.
- **Tinman must stay useful for plain command-line testing**, not only full-screen work.
- **No backlog debt.** Do it, decline it, or name its pivot. **User sharpening 2026-07-27: ask what deferring teaches us.** A pivot is a dependency, so waiting for it changes what gets built: tier-2 markdown testing genuinely cannot be written before `tinman fixture new` exists. A deferral with no dependency under it teaches nothing between now and later, and the work arrives at the later date in the same state, minus the context the person holding it had. Where Captain wants to defer, the test is whether any fact will differ when the item comes back. If none will, that is not caution, it is latency, and Captain is managing its own discomfort.
- Breaking changes are acceptable at 0.1.x, so a correct reshape does not wait for a major version.
- **Specify the floor as well as the ceiling.** A scenario asserting a measurement against a threshold also carries a null control. `all nine` and `all thirteen` are floors, so a glob reading nothing fails rather than passes.
- **Captain runs in the main loop, and the internal roles are dispatched away from it.** Operator preference, 2026-07-27. QM, Crew and Boatswain go out as context-isolated subagents; Captain keeps the human-facing seat and narrates while they are away. Captain never clears the main session in order to become QM.
- **Anything Tinman submits to somebody else's project gets a human review first.** User-stated 2026-07-27, prompted by the tldr-pages contribution and binding on any like it. Evidence from a sandbox strengthens the submission; it never substitutes for the review. Captain drafts and stops.
- **Auto-memory is off**, `autoMemoryEnabled: false` in both settings files. Anything worth keeping goes to a `.feature`, a scantling, `AGENTS.md`, `RIGGING.md`, or here.

## Scheduled work

Every item names its pivot and its role. No "someday".

**Settled 2026-07-27: stream roles are decided by shape.** A program that exits on its own is read as a stream, its complete output unbounded by terminal height; a program still running is read as a screen. For the stream's roles the user chose shape over a blanket rule: any blank-line separated **multi-line** block present makes it a `log` of `article`s, and otherwise it is a `list` of `listitem`s. The discriminator is deliberately multi-line, so a stray blank line between filenames leaves a listing a listing. The deciding argument was the accessibility one: the right role is the one a human would need spoken aloud, and that genuinely differs between `ls` output and a compiler's diagnostics, so a single rule is wrong for one of them in the way a listener notices immediately. Carrying both roles nested was considered and declined against "a role is added when a scenario needs it". Authored as watch11.

**Next harbour, Shipwright.**

- **Constrain locator roles to the TOM role set** in `harness-plan.schema.json` and `driver-protocol.schema.json`. Today `role` is a free string in both, so a plan writing `role: statusbar` validates and then matches nothing, failing at replay time with the operator gone. User decision 2026-07-27: both schemas, at harbour, rather than riding voyage 10. It is a breaking change to two published schemas and needs its own scenario; the existing scantling-enumeration scenarios join the new enums to production with no new machinery.
- **Possible behaviour-stale plank.** `src/driver.rs` `launch` carries `@planks("the failure reports the selection did not reach the {string} named {string}")`. It joins a current pattern so the string join cannot see it, but `launch` is the session-start seam and that step asserts an activation failure. Harbour's coverage triage is the net.
- **`src/driver.rs` reads 0.00% in every tier** while its 24 binding scenarios pass. `tests/cucumber/support.rs:952` SIGKILLs the child, and a killed instrumented process never flushes counters. No coverage claim about the driver means anything until this is closed.
- **`forbidden-doubles` keys on the type names `Mock|Fake|Stub|Dummy`.** `LocalProvider` is a real double, correctly marked, that the rule would not have caught unmarked. Any double named otherwise is invisible.
- **Derive a schema-property-against-production-field check.** A scantling only validates the shape production emits, so a declared property production never emits, or emits and never reads, is untested in both directions.
- **Decide `ratatui-testlib`'s harness layer.** Leaning no: launch must happen inside bwrap and the PTY runner accepts only a prepared process, while `TuiTestHarness` owns spawning and ships no sandbox.

**After the fixture subcommand exists.** Markdown testing tier 2, executing fenced plan blocks, tagged by CommonMark info string. Tier 1, the parse check, landed on voyage 10 and subsumed the bespoke skill-to-parser check. Doc examples elide and need fixtures to be honest, so tier 2 waits on `tinman fixture new`. **mdbook was declined**: `mdbook test` compiles Rust doctests and cannot run a Tinman plan, so the extractor is ours either way; revisit as a marketing call, never a testing one.

**The adversarial-inspection thesis, user-stated 2026-07-27, and it is the sharpest framing the project has.** A `--help` text, a man page, a tldr page and an inference engine all produce *claims*. Tinman does not accept claims; it runs them in the sandbox and reports what happened. That is the categorical difference between Tinman and the projects tldr-pages names on its wall of shame: they republish model output about commands nobody executed, Tinman executes them. It also collapses two requirements into one, since trying arbitrary command lines written by strangers is only sane inside isolation. Sources rank by proximity to the binary: `--help` is ground truth from the binary in hand, `man` opportunistically where the sandbox has one, tldr as a hint. Nothing outranks what the binary says about itself. Authored as watch11.

**Tinman is a testing tool, not a command explainer.** User framing 2026-07-27, and it decided the shape of watch12. An explainer tells the operator what a command does and is believed or not; a probe that writes a plan hands them evidence they commit, re-run, and watch go red the day the program changes under its own documentation. So the product of running documented examples is a plan, not a verdict. Captain's first pass had inspection reporting claims as "refused" or "hint", which is explainer-shaped output, and the user's framing is what caught it. The two phases are unchanged and reached from a new starting point: documentation supplies the hypotheses where a human supplied them under `record`, and replay stays deterministic either way. **Only an honoured example earns an expectation**, since writing a refused one as passing manufactures this project's signature fault and writing it as a failing expectation hands over a permanently red plan describing someone else's stale docs.

**The wall of shame is not what Captain assumed.** Checked rather than recalled: `https://github.com/tldr-pages/tldr/wiki/Clients` lists sites "using LLMs on TLDR pages without crediting the project", with output "often inaccurate and riddled with LLM hallucinations". It is not about hammering servers or client-spec breaches. Tinman's exposure is therefore real and specific, and two things answer it, both authored on voyage 10: the pages are CC-BY-4.0 so credit rides in the written plan the operator commits, and no page text reaches an artifact on the page's authority, because the deterministic pass refuses any name the screen does not independently carry. **Tinman is also not a tldr client** and will not become one: the client specification carries required flags, platform resolution, language handling and cache maintenance, all off-mission, so Tinman asks the operator's installed client instead and keeps no cache of its own.

**Outbound, blocked on project age rather than on us.** CONTRIBUTING sets the bar: "The project has been maintained for at least a year before submitting a page for it," waived for notable projects at maintainer discretion. Tinman is 0.1.x, so **the next release is too early** and the item re-files against a later one. Two things to have ready by then, both cheap: the page itself in their house style, and the honest framing for the PR body. Their AI policy closes pull requests "suspected of being made in whole or in part through generative AI... without human-review," because such output is "often inaccurate" - and a page whose every example was executed in a sandbox before being written is the strongest available answer to that objection. It is an argument to make to humans in a pull request, never a licence to automate the submission. **User ruling 2026-07-27: a human reviews the page and the pull request body before submission, in every case and with no exception.** Sandboxed evidence raises the floor on accuracy; it does not stand in for the review their policy asks for, and a project whose whole thesis is that a claim gets checked before it is repeated does not submit unreviewed text to somebody else's repository. Draft it, hand it over, wait. Contribute a `tinman` page to tldr-pages, `https://tldr.sh`. User direction 2026-07-27: use and incorporate tldr in both directions, so that Tinman is visible there and so that Tinman leverages what is there. The consuming direction is authored on voyage 10 as watch10; the contributing direction is a pull request to the tldr-pages repository and belongs with a release rather than with a voyage, since a page describing an unreleased command surface is the drift problem pointed the other way. Write it after `man` and `completions` land, so the page describes the seven-command surface.

**Undecided, no pivot yet.** A clap-derived machine-readable command manifest, which is what an external coding agent should read instead of parsing prose help. Waiting to see whether the subcommand set grows enough to describe.

**Next release, whoever ships.** `sandbox-spec.schema.json` lost the `fixture` values from `home` and `env.*.from` at the 2026-07-25 harbour, a breaking change to a published schema, so the release carrying it owes a version bump. The bump still has no owner in doctrine; see the gaps below.

**Held, not scheduled.** `jsonschema` 0.48.5 and `generic-array` 0.14.7 sit behind latest with 0 semver updates available. The `locked` policy holds them until a spec or a Captain decision moves them.

## The false-green pattern. The most expensive thing this project has learned.

Every instance was green while asserting nothing. **The test: when a scenario goes green early and cheaply, ask what it would take to make it red. If nothing would, the scenario is the defect.**

The older ones, in brief: a perturbation discharged by deleting its line; a `menu` role the deterministic pass already produced; a launch answering `ok:true` before the program started; a recorded plan carrying no expectation; two activation scenarios asserting text the fixture drew anyway; a scantling that is valid JSON and an invalid schema; a `Given` that seeded nothing, leaving its `Then` asserting against ambient state.

The ones still worth reading:

- **The gplint glob.** `features/**/*.feature` matches zero files, because `**` must consume at least one directory segment and `features/` is flat. It exits 0 having linted nothing. Working values are `features/**` and `features/*.feature`.
- **A plank string shared across six seams.** `@planks("a process is prepared and launched")` is carried by `process.rs`, `sandbox.rs` three times, `bwrap.rs` and `pty.rs`. The string join is green no matter which of the six is dead.
- **An optional schema property is an attestation that cannot fail.** Read the `required` list, not the property list, when asking whether an attestation binds. This is why `cursor` is required in `assistant-ui.schema.json` while it is optional in `tom.schema.json`.
- **A schema can promise a capability production silently discards.** `harness-plan.schema.json` declared `cwd` on a `run:` step while `RunForm::Full` had no such field, and serde dropped the key without a word.
- **A `required` property proves a field exists. Nothing proves it is read.** Crew added the field, left it `None` everywhere, read it nowhere, and threaded a sidecar anyway. Closed by `requiredReferences` in `pty-sandbox-boundary.json`.
- **`tinman replay` was declared and did nothing**, swallowed by a wildcard match arm. **A wildcard match arm is the production form of a glob that reads nothing.** Closed by `command-dispatch-completeness.json`.
- **An absence assertion disarmed by a change to the asset it names.** Captain grew an asset from one line to two and a contiguous-string absence assertion could never fail again. **Absence assertions are the ones that go quiet, because nothing about their output changes when they stop testing.**
- **A focused run against a scenario name that no longer exists selects zero scenarios and exits green.** **A focused run reporting `0 scenarios` is a failed selection, never a pass.** Voyage 10's watchbill was generated by extracting scenario names from the feature files rather than typed, for this reason.
- **A refusal scenario green because the command does not exist.** `completions refuses a shell it cannot emit for` passes today: `tinman completions klingon` exits non-zero because `completions` is not a command at all, so the scenario reports complete while asserting nothing about a refused shell. QM caught it and named it incidental. **Re-run it once the command exists**, or it is green through the whole voyage that builds the thing it claims to test.
- **`mandoc -T lint -W error` exits 0 on an empty file.** Captain hit this live: `tinman man` did not exist, the redirect wrote zero bytes, and the lint reported clean. A validity check with no non-empty floor passes hardest on nothing at all. Closed by `And the emitted page is not empty`.
- **An isolation assertion whose target never reached for the secret.** `record launches its target inside the sandbox` asserted a recorded snapshot did not carry `TINMAN_SECRET` while the fixture never tried to print it. Green for the whole time record ran unsandboxed. Rewritten on voyage 10 with the target printing a marker beside its attempt.

**Corollaries.**

- **Zero coverage on a negative-control branch is the control working, never evidence of dead code.** Captain read `pty::launch` at count 0 and ruled it dead; Shipwright refused and was right. Before removing any zero-count code, find the assertion that would go quiet with it. **`pty::launch` and `pty::capture` are both kept**; a future harbour re-deriving "dead code in `src/pty.rs`" is repeating a settled decision.
- **A scantling only ever tests the shape production emits.** A property production never emits is untested; a property it emits and never reads is also untested, and that one survives a `required` list.
- **Where durable context changes and no scenario reddens, write the scenario that reddens.** Reach for a perturbation only where behaviour genuinely cannot be pinned.
- **A ruling that widens what one layer owes narrows what every other layer may claim.** Re-read every scenario counting on the definition in the same pass. Voyage 10 moved three counts: proof contracts four to five, versioned URIs fifteen to sixteen, dialect-declaring scantlings unchanged at ten. Verified by command before writing them.
- **A green from a subagent is a claim until a command answers it.**

## Rules of the language and the tooling

- **A `Background:` belongs to the `Rule:` it sits under.** Adding a `Rule:` above a `Background:` orphans it for every scenario after. This reddened seven scenarios once. No feature file does it now.
- **gplint's `indentation` rule is off deliberately.** It expects scenarios to nest one level under a `Rule:`. The house style is flat 2/4 throughout.
- **gplint's dupe-name option is `in-feature`, not `in-file`.** One Feature per file makes them the same thing here.
- **Read the registry before naming a version.** `cargo search <crate>` answers it in one command. A `curl` to crates.io from this VM returned nothing for three crates that all exist; `cargo search` found every one. Trust the CLI, not the socket.
- cucumber-rs makes `--tags` and `--name` mutually exclusive. Tag exclusion rides `CUCUMBER_FILTER_TAGS`; `--name` selects the scenario. Encoded in `RIGGING.md` `focused`, so the anchored name does the tag-exclusion job.
- Runner is `tests/cucumber.rs`, `harness = false`, `fail_on_skipped()` so undefined steps redden.
- No clean cucumber-rs dry-run, so `discover: none`. To prove specs parse, run the default tier and read the feature count.
- Read the `result` field before trusting a weather line: `101` is a cargo test failure, so a fast line with `result: 101` is an early abort wearing a duration.
- **Count the directory rather than trusting a report's prose.** Two roles independently called `scantlings/verification-conformance` a three-rule set when `ls` reports four.
- Env confirmed: rustc 1.97 (edition 2024), bwrap 0.11.2, user namespaces enabled, node 22.23.1. `.env` holds a live `TINMAN_API_KEY`, git-ignored, so `@inference` is runnable and costs money per run.

## Design decisions that bind (user-confirmed)

- **The assistant never gains capabilities. Tinman gains subcommands, and the assistant proposes them.** "Help me build a fixture" is not the assistant writing files; it is Tinman growing `tinman fixture new`, proposed and confirmed through `parse_command_line` like every other action. The blast radius of a confused model stays the enumerable set of subcommands, and every capability remains usable by a human with no model in the loop.
- **One surface, three consumers: a human at a terminal, Tinman's own assistant, and someone else's coding agent.** Nothing is assistant-only, so discoverability is the whole game. `SKILL.md` is the one prose source, read directly by agents and rendered for humans. `mandown` was declined: rendering the skill as roff puts it on the drift-prone side.
- **The skill and the man page are not one document.** A man page is exhaustive reference, a skill is teaching material. What unifies them is their sources: the CLI surface from clap, the concepts from prose.
- **`--help` is ground truth** for a program under inspection, from the binary being tested, at the installed version, inside the sandbox. **`man` opportunistically**, absent from a minimal bwrap sandbox. **tldr, including `tlrc`, as hints only**, never as the basis for an assertion, on the drift argument alone; the network objection was wrong and the user caught it, since network is denied to the target and not to Tinman. Voyage 10 authored the consuming half against inference naming, where a stale page costs a worse suggestion the deterministic pass then refuses. The `--help` ground-truth half is still unauthored and rides with the `inspect` stream-versus-screen answer Captain owes.
- **Every driver call except `launch` names its session, and an omitted session is an error rather than a default.** A driver holds several sessions at once, so defaulting is a guess.
- **Tinman is a driver, not only a CLI.** `tinman driver` speaks newline-delimited JSON-RPC 2.0. The YAML plan stays canonical for recorded flows. The protocol is RPC, not a second test format.
- **TOM is the DOM equivalent and inference is codegen.** The deterministic builder is the spine and produces every addressable role. The LLM engine is a second producer of the same shape, capture time only, proposing **names, never roles**.
- **Resolution and confirmation are two operations.** Resolution answers what a locator matches and reports ambiguity as ambiguity. Confirmation runs at capture time only.
- **Terminal size is a property of the run, never of the plan.**
- **Plan YAML grows with the test.** Shorthand removes typing, never adds capability, never weakens a default. An omitted `sandbox:` block means secure defaults, not no sandbox.
- **`home: fixture` and `env.from: fixture` are struck.** `mounts` is the live provision for reaching a tree outside the bound set.
- **The `@inference` tier asserts our seam, never the model's compliance.** Shaping the request is not validating the response.
- **Help text is an asset, not a scenario.** Copy lives in `assets/help/`.
- **Inference: any OpenAI-compatible provider, OpenRouter the default.** `TINMAN_API_KEY`, `TINMAN_BASE_URL`, `TINMAN_MODEL`. Environment wins over `.env`. The credential is vendor-neutral by name so the default is never lock-in.
- **Tier placement.** `@sandbox` marks scenarios that need a real sandbox to run at all, which after voyage 10 includes every `record` and `inspect` scenario that launches. `@inference` is real paid calls, never on the inner loop. Retag a scenario rather than weaken a tier policy.
- **Emulator is `alacritty_terminal` 0.26.0.** `vt100` was unmaintained and failed cell addressing and reverse video. `wezterm-term` is better and disqualified: not on crates.io, and we publish. `alacritty_terminal` is 0.x and breaks across releases; budget for that.
- **Scantlings go into the assistant context, features do not.** Measured: `features/` 94794 chars, all scantlings 37960, the six user-facing ones 22087, `SKILL.md` 6025. The argument that decides it: a scantling cannot drift, because a `@contract` scenario pins it, while the skill can and did.

## Two doctrine gaps worth raising upstream

1. **A release version bump has no owner.** The write-scope list gives Shipwright the manifest for dependency work only, and Shipwright is a harbour role while a release is mid-outbound. Captain bumped `Cargo.toml` to 0.1.2 under Captain's authority at sea, recorded as a departure. Boatswain named the same gap independently.
2. **An adoption proof leaves no durable trace.** A green looks identical proven or not. Every proof so far was made in a session since cleared, so the tree carries no record that any check can fail. The run record has a slot-shaped hole here: it records that a target passed, never that it was ever observed to fail.

## Prior art and the accessibility angle

`ratatui-testlib`, crates.io, MIT, v0.1.0, single maintainer, framework-neutral despite the name. It occupies Tinman's transport layer via the same `portable-pty` and carries no semantic model, no sandboxing, no replay, no inference. Snapshot testing was considered and rejected: `insta` asserts "same as last time", which is drift detection rather than specification.

**Accessibility is the TOM's second consumer**, and after voyage 10 it is closer to being the first. Terminal accessibility is an empty category: VTE exposes a flat `AtkText`, Windows Terminal a UIA `TextPattern`, so an ncurses menu reaches a screen reader indistinguishable from a paragraph. A black-box model deriving ARIA roles, accessible names and now computed presentation from an attributed cell grid is the missing piece. Two free consequences: a wrong role passes a test but a listener notices at once, so holding the model to what a human would need spoken aloud improves it for testing; and never route the TOM through a lossy intermediate, because attribute-derived structure needs a genuine attributed grid.

Keep the framework-neutral naming: `assets/skill/SKILL.md` says "CLIs and full-screen TUIs" and names no framework.

## Reference docs, not sanctioned artifacts

`intent/idea.md`, `intent/isolation.md` and `intent/help.md` are historical intent sources, each carrying a banner saying so, and the whole `intent` directory is in the crate's `exclude` list. Do not treat them as requirements and do not report their staleness. Two scantling descriptions cite them as the source of an invariant, `assistant-command-boundary.json` and `pty-sandbox-boundary.json`, which is provenance rather than a live reference.

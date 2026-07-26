> STOP. Captain's notes: non-binding. Captain writes, Captain trims. Anyone else: close this file now.

# Captain Notes

Binding behaviour lives in `.feature` specs and referenced scantlings. History lives in git. These notes carry only what the next cycle needs.

## Prior art: ratatui-testlib

Checked 2026-07-25. `ratatui-testlib`, crates.io, MIT, first published 12 June 2026, v0.1.0 MVP, single maintainer `beengud` at `github.com/raibid-labs/ratatui-testlib`.

Despite the name it is framework-neutral: `ratatui` is an optional dependency behind a `ratatui-helpers` feature. Core dependencies are `portable-pty`, `termwiz`, `vtparse`. `TuiTestHarness` spawns any terminal program through `CommandBuilder` and asserts against the emulated screen.

It occupies Tinman's transport layer, and it uses the same `portable-pty` crate; the emulator differs, `termwiz`/`vtparse` against our `vt100`. What it does not carry: any semantic model, assertions are `text_at`, `cursor_position` and sixel bounds against raw cells; no sandboxing; no replay; no inference. So the commoditized part is plumbing Tinman would never have differentiated on, and the TOM, isolation-by-default and deterministic replay are untouched.

**Open decision, reopened by the user on 2026-07-25 after an earlier note closed it too fast.** "Not a dependency candidate" was the wrong shape of answer, because it collapsed two separable questions:

1. **Its harness.** Adopting the spawning and driving layer fights the architecture: launch must happen inside bwrap, and the PTY runner accepts only a prepared process, per `AGENTS.md`. `TuiTestHarness` owns spawning through `CommandBuilder` and ships no sandbox. Leaning no, on architecture rather than on maturity.
2. **Its emulator choice, `termwiz`/`vtparse` against our `vt100`.** This is the live question and it is separable from the crate entirely: `termwiz` can be adopted directly. The TOM is derived from emulated cells, so emulator fidelity is foundational. Wide and CJK characters, combining marks, and styling drive region geometry and role heuristics, and a mis-celled screen yields a wrong model that no scenario above it can detect.

User constraint: adopt only if well maintained. That gate probably splits the two. `ratatui-testlib` is six weeks old at v0.1.0 with a single maintainer, which likely fails it for a foundational layer whatever the code quality. `termwiz` is WezTerm's engine and likely passes; unconfirmed, and a study is in flight to settle both with evidence rather than priors.

Timing: this is harbour work, not mid-voyage. Voyage 2 is building `src/screen.rs` and `src/tom.rs` right now, and swapping the emulator underneath that is the opposite of the simplest sufficient change. Decide at the next harbour, on the study's evidence.

Positioning note, unchanged: a crate named for one framework under-indexes for general TUI testing, so Tinman's framework-neutral naming is an advantage to keep. `assets/skill/SKILL.md` already says "CLIs and full-screen TUIs" and names no framework; keep it that way.

Snapshot testing was considered and rejected in the same pass. `insta` asserts "same as last time", which is drift detection rather than specification: a snapshot passes whatever it recorded, including a regression recorded before anyone looked. That contradicts the falsifiable-scenario line the project runs on. The existing `TestBackend` use at `tests/cucumber/support.rs` is the right depth.

## Accessibility is the TOM's second consumer. Noted 2026-07-25.

Not scope, and not a promise. A standing reason to hold the model honest, plus a positioning asset worth stating when the moment comes.

Terminal accessibility is an empty category and the frameworks say so. VTE exposes a flat `AtkText` with no children; Windows Terminal exposes a UIA `TextPattern`. Both model a terminal as text rather than as a control tree, so an ncurses menu reaches a screen reader indistinguishable from a paragraph. Ratatui's a11y issue is open and unassigned, its maintainer stating that no terminal UI framework in any language has found a reasonable path. Textual's and Bubble Tea's equivalents have been open since 2023. Every system that ever succeeded needed the application to cooperate: emacspeak speaks Emacs structures, FrankenTUI requires the app be written in FrankenTUI.

A black-box model deriving ARIA roles and accessible names from an attributed cell grid is the missing piece, and after the 2026-07-25 alignment the TOM already emits that shape. AccessKit is the cross-platform accessibility-tree abstraction with macOS, Unix and Windows adapters and a `Terminal` role, and no TUI has ever wired to it. Ghostty's accessibility work names AI screen readers as its consumer, so the field is drifting this way already.

Two consequences for how the model is built, both free:

1. **Accessibility is a forcing function for honesty.** A wrong role still passes a test, because the locator binds and the run goes green. A person listening notices at once. Holding the model to what a human would need spoken aloud improves it for testing rather than taxing it.
2. **Never route the TOM through a lossy intermediate.** BRLTTY's AT-SPI driver rebuilds a grid from an accessibility hierarchy and fills only text, never colour, silently disabling every attribute-based command. Attribute-derived structure needs a genuine attributed grid, and a terminal is the one place that hands you one. That is the structural advantage over every GUI-derived approach.

**AccessKit: map, do not adopt. Settled 2026-07-25 after checking the crate rather than the impression.**

Three findings against taking its node shape into `scantlings/tom.schema.json`. `NodeId` is documented as a stable identity assigned by the provider, so it is a handle rather than a re-derivable key, and adopting it would import the one thing the locator design rejects. `TreeUpdate` is an atomic change to a tree, so the model is push-based and incremental, while Tinman derives a whole snapshot from a frame and knows nothing about deltas. And AccessKit is a crate at 0.24.1 on 0.x semver with no published JSON Schema, `schemars` being an optional feature rather than a released artifact, so a scantling based on it would transcribe one project's Rust enum rather than cite a specification.

The payoff people reach for is platform bridges, and that payoff arrives at the bridge rather than at the model. Emitting AccessKit nodes needs an adapter walking the TOM, synthesizing node identity at that boundary where a handle is correct. It does not need the durable artifact to carry AccessKit's shape.

Aligning to ARIA already bought the interoperability: AccessKit's roles are ARIA-derived, so the mapping is close to mechanical. Paying a second time in coupling would buy nothing.

The general rule this settles, worth applying to the next candidate: **prefer specifications over implementations.** ARIA, JSON-RPC, JSON Schema and AccName outlive any library. A well-maintained crate is a sound dependency for code and a poor foundation for a durable artifact, because the artifact is meant to survive the code being replaced. `alacritty_terminal` can be swapped; a schema shape cannot.

## Emulator: migrate vt100 to alacritty_terminal. Decided 2026-07-25.

User directive: do not carry technical debt this early; if `vt100` is debt, take something simple, current and well maintained. Differential probing against `alacritty_terminal`, `wezterm-term` and `avt` settled it on evidence rather than reputation.

`vt100` 0.16.2 fails two of the four requirements. Cell addressing: HVP, HPA, REP, CBT and DECAWM are unimplemented, so `ESC[2;3f` lands at the wrong cell and `ESC[?7l` wraps to the wrong row. Reverse video: a wide-character continuation cell is cleared to `Attrs::default()`, so a highlight spanning CJK or emoji reads back striped rather than continuous, which breaks selection detection against agents that emit emoji constantly. Upstream has zero 2026 commits, three open correctness issues and six open correctness pull requests, all concentrated on those same two requirements. One PR author's rationale names our target class directly: OpenCode rendered as "a complete mess" without HPA and REP.

`vt100-ctt` is eliminated on evidence, not suspicion. Its entire diff against upstream is a rename, a `vte` compatibility bump and a new ratatui adapter; it is byte-identical to `vt100` across all nine probed defects. Its default features also pin `unicode-width` incompatibly with our `ratatui 0.30.2`.

`wezterm-term` is the best emulator tested and is disqualified for one decisive reason: it is not published on crates.io, and Tinman is published, so a git dependency makes `cargo publish` fail. `libghostty-vt` needs a Zig toolchain. `avt` has no combining-mark support at all.

`alacritty_terminal` 0.26.0 satisfies all four requirements, is strictly better on cell addressing and reverse video, takes raw PTY bytes without owning a terminal, and costs about fifteen lines of adapter boilerplate. Released 2026-04-06 inside a 65k-star project with 444 contributors and a changelog that marks breaking changes. It is 0.x and does break across releases; budget for that.

Two obligations ride with this decision:

1. **The migration's own regression risk is pinned first.** On the full-row highlight idiom, reverse video then text then erase-to-end-of-line, `vt100` and `wezterm-term` mark the whole row while `alacritty_terminal` marks only the written cells. Migrating blind would under-report exactly the selections we are fixing. `features/virtual-screen.feature:a row highlighted by erasing to end of line reads as reversed throughout` pins the behaviour a user sees, so it binds whichever emulator sits underneath.
2. **Our own adapter is the worse defect and is emulator-independent.** `src/screen.rs` joins each row with `concat()` over blank cells held as empty strings, so column gaps collapse and a screen showing `a     b` reports `ab`. `parse()` also discards `is_wide()` and `is_wide_continuation()`, leaving a wide character's second column indistinguishable from a blank. Every full-screen program positions by address, so `contains()` and `contents()` currently misreport the screen, and every region boundary the TOM derives from them lands in the wrong column.

**The install landed and the migration did not. Perturbation planted 2026-07-25.** Shipwright put `alacritty_terminal = "0.26.0"` in `[dependencies]`, and `src/screen.rs` still calls `vt100::Parser`. Both crates sit in the graph; only `vt100` is imported. Obligation 2, the adapter defect, was fixed on `vt100`, so the four scenarios written to drive the swap went green without it: they proved the column-gap and wide-cell fix, not the emulator change. Obligation 1 makes it worse, because `vt100` satisfies the erase-to-EOL row highlight natively while `alacritty_terminal` marks only written cells, so the migration reddens a currently green scenario rather than greening a red one.

**The perturbation was struck and the migration stranded a third time. Confirmed 2026-07-25 by command, and the mechanism is now abandoned.** QM reported watch2 green 7/7 with the perturbation removed. `grep` finds no `PERTURBATION` token in `src/`, and `src/screen.rs:49` still reads `let mut parser = vt100::Parser::new(ROWS, COLS, 0)`. Both crates remain in `Cargo.toml`. Crew took the cheapest legal fix, which is deleting the statement, exactly as the watch3 ordering note predicted a merged watch would allow. Splitting the watches did not prevent it, because the seam's own scenarios never required the swap.

**Root cause, and it was Captain's fault rather than Crew's.** No scenario in `virtual-screen.feature` fails on `vt100`. CUP, column gaps, wide cells, reversed video and erase-to-EOL all pass, the wide-cell case via the adapter workaround at `src/screen.rs:68`. The migration was never a failing target, so the perturbation was the only pressure on it, and a perturbation is discharged by deleting one line. A perturbation asks Crew to rebuild a seam whose behaviour is already satisfied; it cannot express "satisfied by the wrong dependency".

**The replacement is ordinary failing verification.** Four scenarios added 2026-07-25 pinning sequences `vt100` does not implement: HVP, HPA, REP, and DECAWM autowrap. Watch1 carries them beside all six existing virtual-screen scenarios and `terminal-view.feature`, so the regression risk is pinned in the same watch as the change that creates it.

**Both halves verified against crate source, 2026-07-25, not against the earlier probing summary.** The prior note's claim that `vt100` fails "HPA" was imprecise, and the imprecision was nearly fatal to the fix.

`vt100` 0.16.2 `src/perform.rs` `csi_dispatch`: `'G'` maps to `cha` and `'d'` maps to `vpa`, so CHA and VPA are honoured. Absent from the match arms, falling through to `unhandled_csi`: `'f'` HVP, `'b'` REP, `'Z'` CBT, and backtick HPA. `decset` in `src/screen.rs` has no `[7]` arm, so DECAWM is unsupported and the emulator always wraps.

`alacritty_terminal` 0.26.0 routes through `vte` 0.15 `src/ansi.rs`: `('H',[]) | ('f',[])` both reach `goto`, `('G',[]) | ('`',[])` both reach `goto_col`, `('b',[])` is REP, `('Z',[])` is `move_backward_tabs`, and `NamedPrivateMode::LineWrap` carries DECAWM in `term/mod.rs`.

**So HPA had to be pinned by its true final byte.** CHA and HPA are one arm on `alacritty` and two different fates on `vt100`. A step reaching for `ESC[12G` would have gone green on both emulators and pinned nothing, which is the same false-green that let the perturbation discharge. A second `Rule:` in the feature carries that distinction so QM cannot resolve it the wrong way.

CBT is also unimplemented on `vt100` and is deliberately not pinned: three discriminators plus HPA are sufficient, and a fourth adds no pressure the migration does not already carry.

Expect the erase-to-EOL scenario to need adapter work rather than a straight port: `vt100` marks the whole row, `alacritty_terminal` marks only written cells. It pins what a user sees, so it binds whichever emulator sits underneath.

Do not plant a fourth perturbation here. If Crew hand-implements the four sequences on `vt100` rather than switching, the scenarios go green and the user-visible behaviour is correct; the dependency choice is then a maintenance question for harbour, not a behaviour question, and `vt100` staying unmaintained is the argument to make there.

**Both are recorded during the migration, deliberately.** The 2026-07-25 refit struck `alacritty_terminal` and recorded `vt100` in its place, reconciling the record toward the manifest. That inverts the Rigging read contract: Captain records the selection and installation follows it, so the record leads the manifest and a recorded-but-absent dependency is an install order, never a stale entry to strike. Restored, with `vt100` recorded alongside it because it is genuinely installed and stays until `src/screen.rs` migrates. Removal order: install `alacritty_terminal`, let watch1's virtual-screen scenarios drive `src/screen.rs` off `vt100::Parser`, then strike `vt100` once nothing imports it.

## Voyage 2 — everything remaining from idea.md, isolation.md and help.md

User directive: take the whole remaining scope in one voyage, no backlog debt. See the QM section below for the live watchbill; the original 12-watch numbering is dead.

Voyage 1 shipped the capture spine: sandbox launch, PTY, virtual screen, Ratatui view, key recording, interaction log. Those 11 feature files are untouched and stay out of the watchbill.

The three methodology-conformance scenarios are green and struck from the bill. They ride the `@logic` sweep now, so every later watch still runs under live plank-form, perturbation-quiescence and forbidden-doubles checks.

**The lesson from the ordering that failed.** These scenarios were held below the emulator watch so that quiescence could not be discharged before the rebuild. That reasoning was sound and insufficient: Crew struck the perturbation from inside the emulator watch itself, because nothing in that watch required the swap. Watch ordering cannot substitute for a failing target. Where durable context changes and no scenario reddens, write the scenario that reddens; reach for a perturbation only where the behaviour genuinely cannot be pinned.

## The deterministic model produces every addressable role. Ruled 2026-07-25.

QM raised this as contradictory product intent. It is not: the durable artifacts already decided it, and only the implementation had not caught up. One spec did contradict them and is now fixed.

**What QM found.** `src/tom.rs` `build()` constructs only `Application`, `List`, `Listitem`, `Region` and `Status`. `Menu`, `Menuitem`, `Button` and `Textbox` are reachable solely through `Role::from_name`, the hand-built and engine-reply path. Yet `inspect-command.feature` asserts a `menuitem` named `Settings` on the default tier, which excludes `@inference` and has no credential, and `driver-session.feature` activates the same item.

**The ruling: the deterministic builder derives `menu`, `menuitem`, `button` and `textbox` from terminal idioms.** Three durable artifacts force it and none of them is in tension. `assets/examples/settings-flow.yaml` is the canonical plan and addresses `role: menuitem`, `label: Username` and `role: button`. `features/replay.feature:replay performs no inference` is a binding scenario. The `Rule:` at `tom-inference.feature:8` states replay rebuilds the model with no model invocation, and the `Rule:` at line 6 states a hand-authored plan needs no model at all. A locator a plan carries must therefore bind against the deterministic model alone. Were these roles inference-only, the canonical plan could never replay, which removes the product's reason to exist.

The capture-time round trip already assumed this and is the strongest evidence: every `tom-inference` scenario validates a proposed locator against the deterministic model and falls back when it will not bind. A locator that reaches a plan is deterministically bindable by construction.

**Three scenarios added to `terminal-object-model.feature`**, pinning the menu bar, the bracketed button and the labelled input field, with a `Rule:` carrying why. They ride watch1 beside all seven existing TOM scenarios and the four locator scenarios, because they change `build()` and the existing reads are the regression risk that change creates.

**One scenario was genuinely wrong and is replaced.** `inference names a region the deterministic pass left unnamed` had an engine label a top line of words a `menu`. Once the deterministic pass reads a menu bar, that scenario passes whatever the engine returns: a false green asserting nothing. It is now `inference refines a role the deterministic pass cannot distinguish`, where an engine labels a bordered pane of lines a `log`. Geometry cannot separate `log` from `list`, so only inference can supply it, and `an unavailable engine leaves the deterministic model standing` is already its null control, asserting the same shape reads as `list` with no engine.

**This also unblocks QM's second blocker.** The fixture terminal program's content is now fully determined by durable artifacts: a menu bar carrying `Settings`, a `Username` input, a `Save` button, and a scrolling pane. QM owns building it; nothing further is owed from Captain.

## Five rulings on QM's third-pass blockers. 2026-07-25.

Watches for the terminal object model, inference and the inspect and test commands all went green after the role ruling above, 14/14, 17/17 and 6/6. These five settle what QM raised next.

**1. Record writes one file and it is a plan.** `record-command.feature` asserted a written interaction log of `command` and `events` and then replayed the same `tinman.yaml` as a plan, which cannot both hold. `assets/help/tinman.txt` already settled it: record "capture[s] a live session into an editable plan". The interaction log keeps its own artifact and `features/interaction-log.feature` owns it, so the scantling stays referenced. The record scenarios now say plan throughout.

**2. The output path is `--output`, defaulting to `tinman.yaml`.** No durable artifact named a CLI form and QM correctly refused to invent one. Added a scenario pinning the option, and the overwrite-refusal scenario now uses it too. The top-level help advertises commands rather than per-command options, so `assets/help/tinman.txt` needs no edit and `every option the help text advertises is accepted by the parser` stays honest.

**3. Terminal size is a property of the run, never of the plan.** The caller supplies it, defaulting to the operator's terminal, and it reaches the PTY and the virtual screen together. A plan recording its capture size would invite replay to restore that size, which is the one thing the width scenarios exist to prevent. `src/screen.rs` pinning 24x80 is now an ordinary Crew target, and `expect.within`, blocker 4, follows it.

**4. Resolution and confirmation are two operations.** QM found one seam being asked to answer `Ambiguous(2)` for `tom-locators` and `One` for `tom-inference`. Resolution answers what a locator matches as the model stands and reports ambiguity as ambiguity, which is what a replaying test needs. Confirmation runs at capture time only, narrowing by scope or ordinal until one region binds and recording which of `exact`, `scoped` or `ordinal` it needed. Collapsing them makes an ambiguous locator look bindable to the test that must later resolve it alone. That is now a `Rule:` in `tom-inference.feature`, and blockers 5 and 6 become Crew targets.

**5. Inference proposes names, never roles. This corrects my own error.** Ruling the deterministic model produces every addressable role has a consequence I missed one pass earlier: no role can be inference-only, because a plan may address any of them. My replacement scenario had an engine supply `log`, and `semantic-capture.feature` then needed `log` and `article` deterministically, with its own `Rule:` stating inference never runs at capture time. Both cannot hold.

The original scenario title was right all along and only its body was wrong: it said inference *names* a region and then asserted a *role*. It now has an engine name an unbordered pane from its heading line, which is genuinely beyond the deterministic pass, since names come from border titles and an unbordered pane carries none. The name must still be text the screen shows, so `a name the screen does not carry is rejected` keeps its force. A `log` of `article` children is now derived deterministically from blank-line separated entries, pinned in `terminal-object-model.feature`, which unblocks all five semantic-capture targets.

The general lesson, worth carrying: a ruling that widens what the deterministic layer owes narrows what any other layer may claim, and the scenarios asserting the other layer's contribution must be re-read the same pass. I did not, and QM found it one watch later.

## Two more rulings, and interim custody. 2026-07-25.

Watches for the terminal object model and locators, tom-inference, and record and replay all went green: 36 targets, 39 fresh focused runs. `@logic` reached 122 of 127 in 24s against the 120s budget; `@sandbox` rose from 2 to 8 of 25. Plank join clean at 150 planks, `ast-grep` conformance clean.

**6. The semantic-capture window could not hold what it asserted, and that was my arithmetic.** Its Background held 12 messages in a 5 line window while `capturing the visible scope reads only the current window` asserted 5 visible `article` items. Ruling 5 had just defined an article as a blank-line separated entry, so 5 articles need 9 lines: 5 entries and 4 separators. The window is now 9 lines and the two agree. This is the second time a role definition landed without re-reading every scenario that counts on it; the lesson under ruling 5 stands and I paid it twice.

**7. A launch that cannot execute its program is a failed launch.** Crew observed `/bin/sh: 1: /tmp/tinman-probe/p: not found`, because the bwrap vector binds `/bin`, `/lib` and `/lib64` only, while `launch` answered `ok:true` as soon as `/bin/sh` started. That is a vacuous pass: every later step then asserts against a blank screen and passes while doing it, which is the same false-green shape that let the perturbation discharge and that ruling 5 had to undo. A `Rule:` in `driver-protocol.feature` now states that a launch binds the system directories plus whatever the session's sandbox spec names and nothing more, that `home: fixture` in `scantlings/sandbox-spec.schema.json` is the provision for reaching a fixture, and that an unreachable program fails rather than yielding a session. A scenario pins it, and it rides the `@sandbox` sweep with the rest of that feature.

**Interim custody landed: `190a7ee`, tree clean.** The watchbill was not spent, so this departed from the usual trigger, and QM named custody as owed. Thirty-six verified green targets plus a full dependency refit had sat uncommitted across the whole voyage, and the exposure was no longer proportionate to waiting for a spent bill. 49 files, +3863/-497. **The base commit is now `190a7ee`; `88da55d` is spent.**

Boatswain ran the plank join rather than reading it, extracting 270 step-definition pattern literals and joining by exact string against the inventory: 291 of 291 planks match a current pattern, zero stale, zero malformed, no provisional. No custody foul, no regression. All 23 sweep failures are `Step doesn't match any function`, never an assertion, so the reds are unwritten steps rather than broken behaviour.

**Three findings carried, none of them Captain's to decide:**

1. Five orphaned step definitions in `tests/cucumber.rs`, including two left by ruling 5's rewrite, `an engine that labels that pane a {string}` and `an engine that labels that line a {string}`. QM's write scope; it has cleaned orphans before and re-derives them without help.
2. `pub struct FrontMatter` and `pub struct Skill` in `src/skill.rs` carry no plank. Beyond the diff, so harbour.
3. `step-usage` and `discover` still read `none`, so Boatswain discharged the join with a scratch script this voyage. That is not repeatable custody. Deriving a checker that extracts the `#[given]`, `#[when]` and `#[then]` pattern literals would make it a real project command. Shipwright work at the next harbour, and the strongest candidate there.

## The default tier is fully green. 2026-07-26.

`@logic`: 127 scenarios, 127 passed, exit 0, 31.7s at 64 workers against the 120s budget. Semantic capture spent green with it, and Crew implemented the `capture` seam in `src/driver.rs`. `@sandbox` stands at 8 of 26, every failure an undefined step rather than an assertion, so the remainder is QM authoring work with no blocker in front of it.

QM fixed three verification-support defects that were each silently disarming the sandbox tier: fixtures unreachable inside the sandbox because `args_with_home` binds only `/bin`, `/lib`, `/lib64` and the session home; every `2>/dev/null` redirect failing because the sandbox has no `/dev`, which aborted the scroll reader after one window; and 44 leaked session homes under `/tmp`, now reclaimed at suite start.

**8. Semantic capture is retagged `@sandbox`.** It carried no tag, so it sat in the default tier while its Background launched a real Bubblewrap driver session at 64 workers. The tier policy assigns that to `@sandbox`. This is the case the tier-placement note anticipated: retag the scenario, never weaken the tier policy. `@logic` drops to 122 and `@sandbox` rises to 31.

**9. A stale green outside the watchbill, and the fix is at its root.** `a written plan replays the interaction it recorded` proved nothing: a real record run writes only `tui: {command, steps: [press: q]}`, so the plan carried no expectation and the step asserted only that `flow::execute` returned `Ok`. It would pass identically against an error screen.

Strengthening the step would have treated the symptom. The defect is that a recorded plan carries nothing to verify, which contradicts this feature's own `Rule:` that a recording which cannot replay itself is a draft. `a recorded plan carries an expectation for what the screen showed` now pins the root, and the replay scenario becomes meaningful once plans carry expectations. Both ride watch1.

That is the fourth false green this voyage: the discharged perturbation, the vacuous `menu` role, the vacuous launch, and now this. Every one passed while asserting nothing. The pattern worth carrying: when a scenario goes green early and cheaply, ask what it would take to make it red, and if nothing would, the scenario is the defect.

## Voyage 2 is verified across all three tiers. 2026-07-26.

Watchbill spent, every watch green: `@logic` 123 scenarios and 404 steps, `@sandbox` 31 scenarios and 120 steps, `@inference` 3 scenarios and 8 steps against the real provider. 336 planks all naming current patterns, `ast-grep` conformance clean, `fmt` and `clippy` clean, no `PERTURBATION` standing. QM authored 29 previously undefined steps and fixed four harness defects in its own scope, including a `pty::capture` that hung forever on a fixture whose read loop never exits.

Custody dispatched at base `190a7ee` with the watchbill to be struck.

**Deliberately deferred rather than folded into this voyage.** Both were live when the bill went spent, and taking custody of a fully green voyage first is worth more than closing them a cycle sooner.

**The fifth false green is fixed and voyage 3 is open on it.** It was two scenarios, not the one QM named: `activating a menu item opens what it names` and `activation reaches an item the selection is not already on` both asserted the screen contains `"Username"`. The shared fixture draws `Username: ________` from its first frame and must, because `features/replay.feature:a failure report shows the screen the step saw` and `features/test-command.feature:the failure report names the step and shows the screen` both assert on that text. So neither `Then` could tell an opened pane from an unopened one.

Both now name a `"button"` named `"Save"`, which `assets/examples/settings-flow.yaml` already places inside Settings, so the marker comes from the canonical plan rather than invented for the test. A `Rule:` records why, so a later reader does not revert it toward the simpler-looking text assertion. The two scenarios depending on first-frame text are untouched, which was the constraint.

Watchbill: the two targets by name, then a `@sandbox` sweep as regression cover for the fixture change they force. QM must make the fixture draw `Save` only after activation; that is verification support and its own scope.

**That `Rule:` orphaned the `Background:` and reddened seven scenarios. My error, and worth carrying as a rule of the language.** A `Background:` belongs to the `Rule:` it sits under. `driver-session.feature` had its `Background:` below the first `Rule:`, so opening a second rule started a block that inherited no background, and every scenario after it ran with no session. QM established it by run rather than by reading: under rule 1 the background step prints and passes, under rule 2 it never prints and the session accessor panics.

Fixed by hoisting the `Background:` above the first `Rule:`, which makes it feature-level and restores exactly the reach it had when one rule held every scenario.

**Then I swept for the same shape and found it latent in `semantic-capture.feature`**, Rule at 7 and Background at 9. It was green only because that file has one rule; a second would have orphaned it identically. Reordered, and no feature file now places a `Rule:` above its `Background:`. Adding a `Rule:` to a feature with a background is a structural edit, not a prose edit.

**Closed. The fixture never opened anything, which is why the old assertion could not fail.** QM gave `FIXTURE_TUI` a selection it tracks and redraws in reverse video, and opening `Settings` now draws `[Save]`, which the model reads as a button. Frame 1 is byte-identical to before, so `inspect-command` and the two failure-report scenarios still read the text they assert on: the constraint held.

**QM planted the red, which is what the earlier false greens all lacked.** Neutralizing the selection move left the target red with `the screen shows no "button" named "Save"`; restoring it went green. So the assertion binds to the selection reaching Settings rather than to an activation merely occurring. A scenario proved this way is worth more than four that were never made to fail.

`@sandbox` 31 scenarios and 120 steps green in 28.9s, `@logic` 123 scenarios and 404 steps green in 24.3s, both inside budget. QM ran the `@logic` sweep unprompted because a support-code edit selects every tier that loads it, not only the watch's own; that is the Planking agreement's support-edit rule applied without being asked.

**Unexplained nondeterminism, recorded not guessed at.** Two `broad-sandbox` sweeps on an unchanged tree with no recompile disagreed, 23 then 24 passing, differing on `activation fails when the selection cannot reach the item`. QM declined to attribute it from one truncated observation and will re-observe once the background fault clears. If it is harness, it is QM's to engineer out; the Verification agreement forbids tolerating it either way.

**`cwd` divergence, harbour.** `scantlings/prepared-process.schema.json` documents a `cwd` field that `PreparedProcess` does not carry; Crew threaded it as a parameter instead, because adding the field means touching struct literals in `tests/cucumber.rs`, which is QM's scope. The conformance scenario is green because the field is optional, so this is latent divergence rather than a live failure. `AGENTS.md` says the PTY runner accepts only a prepared process and never constructs backend arguments itself, which argues the field should land rather than the scantling drop it. Decide at harbour with a scenario to drive it.

**Rigging contradiction worth recording once.** `focused` carries no tag exclusions because cucumber-rs forbids `--name` with `--tags`, established by running it. The Rigging read contract expects the exclusions on every derived command, so this stack cannot satisfy that clause on this one command. `AGENTS.md` already documents the constraint; the contract-level exception is what was missing, and it is recorded here now so a later role stops rediscovering it.

## Design decisions this voyage (user-confirmed)

- **Tinman is a driver, not only a CLI.** Tests live in pytest, jest, bun test and drive Tinman the way Playwright's language clients drive the Node driver. `tinman driver` speaks newline-delimited JSON on stdin and stdout. This is the primary consumption surface; the YAML plan stays canonical for recorded flows. Keeps idea.md's "no programming-language DSL" intact, because the protocol is RPC, not a second test format.
- **TOM is the DOM equivalent and inference is codegen.** Deterministic builder is the spine: geometry from Ratatui-shaped nested rects, roles and names from heuristics. The LLM engine is a second producer of the same shape, capture time only. A hand-authored plan needs no model. This is why replay needs no model.
- **Plan YAML grows with the test.** One canonical model, several surface forms. Shorthand removes typing, never adds capability, never weakens a default. `features/plan-shorthand.feature` pins that both example assets parse identically, and that an omitted `sandbox:` block means secure defaults rather than no sandbox.
- **Help text is an asset, not a scenario.** Per the content policy: copy lives in `assets/help/`, scenarios own only the seams we own. `assets/help/tinman.txt` carries one `{{tagline}}` placeholder that inference fills; `inference-unavailable.txt` and `assistant-prompt.txt` carry the other two operator-visible strings. Inlined at build time.
- **No acronym validation. Settled 2026-07-25, user directive "don't over specify, let inference do whatever it wants give the name and description."** The generator receives the bundled skill's name and description and nothing else; whatever comes back fills the tagline. `features/acronym.feature` dropped from ten scenarios to two: the generated expansion fills the tagline, and an empty generation falls back to the unavailable notice. The six-word rule, the connective rule, the initials-spell-TINMAN check, the punctuation and newline rejections and the retry-once behaviour are all gone. help.md's "exactly six words" is superseded and is no longer a live question.
- **Inference: any OpenAI-compatible provider, OpenRouter as the default. Settled 2026-07-25.** Three configuration values, environment or dotenv, environment winning: `TINMAN_API_KEY` for the credential, `TINMAN_BASE_URL` defaulting to `https://openrouter.ai/api/v1`, `TINMAN_MODEL` defaulting to `deepseek/deepseek-v4-flash`. The credential is vendor-neutral by name so the default choice is never a lock-in. `features/inference-provider.feature` pins both defaults and both overrides plus bearer-token construction. `ureq` for the call, blocking, keeps tokio a dev-dependency. `dotenvy` for `.env`, which is git-ignored.
- **Tier placement.** `@sandbox` marks scenarios whose assertion is isolation itself, matching voyage 1's line; ordinary PTY launches stay default tier. `@inference` is new: real paid provider calls, never on the inner loop. If QM finds a fixture-launching default-tier scenario needs real bwrap, retag it rather than weaken the tier policy.

## Deck state at hand-off, 2026-07-26. Read this first.

**Voyage 2 shipped as 0.1.2. See the release section below for what the publish taught.**

`dc88983` stale-green repair on driver-session activation; `44266d6` driver session verbs, sandbox grants, recorded expectations; `190a7ee` terminal object model, locators, inference, record and replay.

All three tiers green at custody, rerun fresh by Boatswain rather than inherited: `@logic` 123 of 123 in 24.3s, `@sandbox` 31 of 31 in 26.4s, both inside the 120s budget. 336 planks, 174 distinct, zero stale or malformed, zero provisional. Conformance, `fmt` and `clippy` clean. No perturbation stands. Watchbill struck.

**Voyage 3 opened on the three open questions. Operator directed all three, 2026-07-26.**

The `@inference` sweep is watch2, a tier tag, so QM runs the tier unfiltered. It settles the one claim Boatswain judged by reading rather than by running: that the rebuilt fixture's opening frame is byte-identical at `sel=0`, leaving the two `@inference` scenarios in `tom-inference.feature` unaffected. It bills a real provider, which the operator has now authorized.

The other two became watch1, both `@conformance` in `features/methodology-conformance.feature`. They sit under their own `Rule:`, and that feature carries no `Background:`, so adding a rule was safe here; the `driver-session` lesson was checked before writing rather than after.

**Boatswain's framing of the asset gap needed correcting, and the correction narrowed the check.** It reported `scantlings/**` and `assets/examples/**` reached by no gate. The examples are reached: `harness-plan.feature:the example flow conforms to the harness schema` validates `settings-flow.yaml` against the schema, and `plan-shorthand.feature:the shorthand form and the full form parse to the same plan` parses both. Every scantling is likewise loaded by some scenario, so malformed JSON already reddens. The residual is narrower and worse: a scantling can be valid JSON and an invalid schema. A mistyped keyword such as `"type": "strng"` yields a schema that validates everything, so its attestation passes while asserting nothing. That is this project's recurring false green in a new place, and it is what watch1 pins. Eight scantlings declare a dialect; the three boundary contracts carry none, because they are proof contracts discharged by their own checkers.

**Both checks are proven by planted red, and Captain planted them. 2026-07-26.** QM raised this as its one blocker and deferred it to harbour, correctly for its own scope: both checks read Captain-custodied `scantlings/` and `assets/examples/`, and the custody hook denied QM's write with `Shipshape custody: qm MUST NOT write scantlings`. But those are Captain's files, and an unproven check is precisely the false green this voyage has spent itself finding, so carrying it to harbour was the worse of the two costs. Three plants, each reverted with `git checkout` so restoration is byte-identical rather than retyped:

1. `"type": "integer"` to `"intger"` in `tom.schema.json`. Red, naming the file and the pointer `/properties/rows/type`. Green on revert.
2. `@v0.1.2` to `@v0.1.1` in that file's `$id`. Red, naming the URI and the packaged version it failed to match.
3. The `$schema` header deleted from `settings-flow-shorthand.yaml`, taking the URI count from thirteen to twelve. Red, naming the plan that carries no `$schema`.

Plant 3 is the one worth having done. It proves the count guard is load-bearing rather than decorative, which was the argument for writing it; a check that only compares the URIs it happens to find would have passed that plant while a pin went missing.

**Both new scenarios name their count, deliberately.** `all eight` and `all thirteen` are the null control in the form this domain allows: a glob that reads nothing would otherwise satisfy "each one validates" exactly as a full read does. The counts are load-bearing rather than decorative, since a scantling added without a pin is precisely the drift watch1 exists to catch, and the count is what reddens.

**Carried to harbour, none blocking:**

- Five orphaned step definitions in `tests/cucumber.rs`, unchanged since `44266d6`. QM's scope.
- Three planks in `src/inference.rs` naming patterns among those five orphans. The string join stays green because the definitions still exist, so those seams trace to a contract no scenario asserts. This is behaviour-staleness, which no command on this stack can reach.
- **The strongest harbour item: derive `step-usage`.** Three roles in a row discharged the plank join with an ad-hoc script. That is not repeatable custody, and it is the one check that would also catch the orphan drift above. A checker extracting the `#[given]`, `#[when]` and `#[then]` pattern literals closes both.
- `focused` cannot compose the tag exclusions the Rigging read contract asks of every verification command; cucumber-rs answers `error: the argument '--name <regex>' cannot be used with '--tags <tagexpr>'`. The value is correct as written and a role obeying the contract literally gets a hard error. Wants a note in `RIGGING.md`, which is Shipwright's file.
- `cwd` documented in `prepared-process.schema.json` and absent from `PreparedProcess`.
- Two unplanked seams in `src/skill.rs`.
- ~~Unexplained nondeterminism in `activation fails when the selection cannot reach the item`.~~ **Closed 2026-07-26, root-caused rather than re-observed.** QM found a missing readiness gate at `launch_driver_session`: the driver's launch reply says the program started, not that it drew, so a step could read the menu line mid-write with reverse video not yet reset and see every item as selected. Gated on the driver's own `expect` for `READY`, which each fixture draws last. `@sandbox` went 30 of 31 to 31 of 31. It was harness, as suspected, and QM engineered it out.
- No derived check joins a scantling enum to the production enum it constrains. That is how the 17-role `Role` drift against the 11-role `tom.schema.json` went unseen. A plausible `@conformance` candidate, and the last of this shape left standing.

## The release taught two things, and the first was my error. 2026-07-26.

**0.1.1 was already on crates.io and I did not check before recommending the tag.** `cargo publish` refused with `crate tinman@0.1.1 already exists on crates.io index`. The registry carries 0.1.0 at 2026-07-25 11:36Z and 0.1.1 at 12:25Z, both by `dmytri`, neither yanked. 0.1.1 was published from `1daee60`, which predates every commit of this voyage. So the version in `Cargo.toml` had been spent for a day and the whole of voyage 2 was unpublished.

The damage was a tag pointing at a lie: I had already pushed `v0.1.1` at `5b5289e`, which carries voyage 2, while crates.io 0.1.1 carries none of it. The schema URIs pinned into that tag, which is exactly the drift the pinning existed to close, re-created under a new name. Operator ruled: force-move `v0.1.1` to `1daee60`, its true source. Done and pushed, so both names now tell the truth.

**The rule worth carrying: read the registry before naming a version.** A published version is immutable, so the check is cheap and the mistake is not. `cargo search tinman` answers it in one command, and it is the same command `RIGGING.md` already names under `## Outbound` as the verify step. It works as a pre-flight read too.

**Captain bumped `Cargo.toml` to 0.1.2 directly, and that is a recorded departure.** No role owns a release version bump: the write-scope list gives Shipwright the manifest for dependency install and upgrade, which this is not, and Shipwright is a harbour role while this was mid-outbound. Under Captain's authority at sea this is the minimal action that restores progress, so it was taken and is recorded here. Worth raising as a doctrine gap: the release version is an outbound decision with no write scope naming it.

`cargo check --all-targets --offline` refreshed the lock; the diff is one line, our own version, so the `locked` policy held and nothing re-resolved.

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

## Standing preferences the user has stated

These govern how specs are written here and are worth reading before authoring any.

- **Prefer a concise attestation on a scantling over verbose behavioural scenarios**, and audit freshly authored specs against that, not only inherited ones. The tell is one rule restated once per variable: five `inference-provider` scenarios collapsed to two behaviour scenarios plus one `@contract` this way.
- **A scantling based on a well-known standard beats a bespoke one.** Applied so far: JSON-RPC 2.0 for the driver protocol, WAI-ARIA for the role taxonomy, JSON Schema 2020-12 throughout. Prefer a specification over a library's data model, which is why AccessKit's node shape was declined.
- **`Rule:` prose carries durable context only.** Requirements belong in scenarios. Five Rules were trimmed on 2026-07-25 after one stated the whole activation convention with no scenario pinning any of it.
- **Tinman must stay useful for plain command-line testing**, not only full-screen work. A coding agent is both: one prompt non-interactively is a CLI, the same binary interactive is a TUI, and a suite driving it should need one tool. This is why a run step reads pipes and keeps its streams and exit status distinguishable.
- Breaking changes are acceptable at 0.1.x, so a correct reshape does not wait for a major version.
- **Specify the floor as well as the ceiling.** A scenario asserting a measurement against a threshold also carries a null control: an empty or no-op operation measures near zero. Executable specs make behaviour legible and calibration opaque, so a threshold scenario reads as a product claim while the step definition quietly binds a number no reader of the feature can see. The worked case is `ratatui-testlib`, checked 2026-07-25: `assert_render_budget(60.0)` asserts input-to-render latency under 16.67ms while the harness held an unconditional 50ms sleep and a 100ms read timeout, a measurement floor near 150ms, about nine times the threshold asserted. A null control exposes a floor instantly and is writable without knowing the constant exists. Add a metamorphic relation where one fits, such as ten keystrokes measuring about ten times one keystroke. Never instrument a latency measurement across a synchronization primitive we control: mark t0 at the write and t1 at the first response byte, never across our own poll loop. The residual risk this guards is specific to this workflow: the step definition and the production code are authored by the same process, so they can agree perfectly and both be wrong, and the counter-pressure is specifications constraining the shape of the answer rather than restating the implementation.

## Tier budgets: one set, two withheld. 2026-07-25.

`budget: 120s` on the default tier, against an observed 22.3s. That is the one defensible figure the weather carries, and even it is a red run.

The other three weather entries are aborts, not measurements, and setting a ceiling from one would redden the moment scenarios did real work. `@logic` 631ms and 22329ms are the same tier minutes apart, which is the tell. The 11 `@sandbox` scenarios cannot launch real `bwrap` in 424ms, and the refit confirmed no `@inference` scenario reached a real provider call, so 426ms measures three undefined-step failures. `budget-sandbox` and `budget-inference` stay unset until a run in those tiers reaches real work; the floors they need do not exist yet.

Read the `result` field before trusting a weather line: `101` is a cargo test failure, so a fast line with `result: 101` is an early abort wearing a duration.

## Hand-testing found two spec faults, not code faults. 2026-07-25.

The operator ran `tinman --help` on a real terminal. The acronym came back as a block of prose, and the assistant prompt printed and exited. Both are specification gaps, and in both the implementation obeys its spec exactly.

**The acronym prompt carried no instruction.** `skill::acronym_context()` sent the skill's name and its 60-word description and nothing else, because `features/bundled-skill.feature` pinned the context to exactly those two fields. A model handed a name and a description with no instruction returns prose, correctly. `assets/help/acronym-prompt.txt` now carries the instruction and the scenario is renamed to pin it first in the context. The instruction constrains output shape only, one line and nothing around it, so the 2026-07-25 no-validation directive still holds: shaping the request is not validating the response, and no content rule came back.

**Nothing pinned the read loop.** Every `interactive-help.feature` scenario drove `assistant::ask()` as a seam through `the operator asks {string}`, so `main()` printing the prompt and returning satisfied the whole file. Two scenarios added: a question typed at the prompt is answered, and ending the input exits successfully.

**A third gap, from the same session.** The help asset advertises `-V, --version` and `src/cli.rs` declares no version argument, so the flag exits 2. The existing scenario pins that every accepted command appears in the help; nothing pinned the converse, so the asset and the parser drift apart and stay green. `features/conventional-help.feature:every option the help text advertises is accepted by the parser` closes that direction.

All three are watchbilled, watch1 and watch4. Hand-run again 2026-07-25 at the current deck: `tinman --help` exits 0, and both faults are still live. The tagline line comes back empty rather than carrying the acronym or the unavailable notice, with `TINMAN_API_KEY` set in `.env`, so `an empty generation falls back to the unavailable notice` is unimplemented. `--version` exits 2 with `unexpected argument '--version' found`, so `every option the help text advertises is accepted by the parser` is unimplemented. Both wait on QM.

## Auto-memory is off. Discipline instead. Decided 2026-07-25.

User directive: no auto-memory on this project, durable artifacts instead. The mechanism contradicts the method it was running under. Shipshape's first Article is that durable artifacts outrank chat, and the Context bulkhead names an agent memory store as the specific way that Article gets circumvented. A store that re-injects Captain decisions into every role session makes the bulkhead advisory: QM cannot tell which of its judgments came from a spec and which from a remembered decision, and no report can show the difference.

`autoMemoryEnabled: false` in both `~/.claude/settings.json` and this project's `.claude/settings.local.json`. The nine entries were deleted after each was confirmed present in a durable artifact; only the null-control rule needed migrating, and it is now in the standing preferences above. Do not write memories on this project. Anything worth keeping goes to a `.feature`, a scantling, `AGENTS.md`, `RIGGING.md`, or these notes.

**The residual is discharged. Confirmed 2026-07-25 by reading both settings files and this session's own context.** `autoMemoryEnabled: false` in both, and no memory index renders into a fresh session. The earlier QM refusals came from sessions constructed before the flag was set; a restart cleared them, as expected. No operator action stands.

## Open questions

None standing. `.env` holds a live provider credential under `TINMAN_API_KEY`, so watch10, the `@inference` sweep, is runnable. An earlier note in this file claimed that key was empty; that was wrong, corrected by reading the file. `.env` is git-ignored and its value never travels into a spec, a report, or a commit.

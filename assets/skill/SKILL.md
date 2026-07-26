---
name: tinman
description: "Use this skill to drive, record and test terminal applications and coding agents. Tinman is a deterministic black-box testing framework for CLIs and full-screen TUIs: it puppeteers a real program through a sandboxed PTY, reads the screen as a semantic Terminal Object Model, and replays recorded interactions with no model invocation and no network."
---

# Tinman

Tinman is to terminal applications what Playwright and Capybara are to web applications. It drives a real program through an embedded PTY, reads what the program rendered, and binds test steps to semantic roles rather than to cell coordinates. It never inspects application internals.

Two phases, and the split is the whole design:

- **Capture time** may infer. Inference reads the screen and names what it sees, producing an editable plan.
- **Replay time** is absolutely deterministic. No model is invoked, and no network connection is opened.

Sandboxed execution is the default. Tinman is built to run coding agents, which execute arbitrary commands, so the operator's home, environment, PATH and network are hidden unless a plan grants them explicitly.

## What problems it solves

- Testing a full-screen TUI, where there is no DOM, no accessibility tree and no automation API.
- Testing a coding agent end to end, driving it as a person would and asserting on what it produced.
- Turning a live interaction into a repeatable regression test.
- Running any of the above safely, without exposing your credentials or your home directory to the program under test.

## The Terminal Object Model

The TOM is the terminal's document object model: a tree of nested rectangles, following Ratatui's geometry of horizontal and vertical splits, with a semantic role attached to each region. Roles include `menu`, `menuitem`, `list`, `listitem`, `table`, `row`, `column`, `dialog`, `button`, `textbox`, `statusbar`, `message-pane`, `message`, `tree` and `treeitem`.

It is a semantic reading of the rendered screen, not a reconstruction of the program that drew it.

Locators bind to the TOM by role and name, and resolution is mechanical. A test you write by hand needs no inference at all. Use `tinman inspect` to discover the roles and names a program exposes.

## Commands

- `tinman record <command...>` captures a live session into an editable plan.
- `tinman test <plan>` runs a plan and reports whether it passed, replaying it exactly with no model invocation and no network.
- `tinman inspect <command...>` prints the terminal object model of a running program.
- `tinman driver` speaks the JSON driver protocol on stdin and stdout.

## Driving Tinman from your own test suite

`tinman driver` exchanges one JSON message per line on stdin and stdout, so a test in pytest, jest, bun test or anything else needs only a subprocess and a JSON parser.

Request operations: `launch`, `activate`, `fill`, `press`, `expect`, `capture`, `screen`, `tom`, `sandbox`, `close`. Every request carries an `id`; every reply echoes it and carries `ok`.

```json
{"id": 1, "op": "launch", "command": "opencode"}
{"id": 2, "op": "activate", "session": "s1", "role": "menuitem", "name": "Settings"}
{"id": 3, "op": "fill", "session": "s1", "label": "Username", "value": "dmytri"}
{"id": 4, "op": "expect", "session": "s1", "text": "Saved"}
```

A failed action is a reply with `ok: false` and an `error`, not a dropped session, so your test framework reports the failure in its own voice.

## Plans

A plan is YAML, and it is the canonical representation of a recorded flow. It grows with what the test needs. The simplest useful plan:

```yaml
tui: opencode
steps:
  - activate: Settings
  - fill: {label: Username, value: dmytri}
  - expect: Saved
```

The full form drives several processes and states its isolation explicitly:

```yaml
sandbox:
  home: empty
  network: deny
  path:
    - ./fixtures/bin
    - /usr/bin
  mounts:
    - source: ./fixtures/project
      target: /workspace
      mode: copy

flow:
  - run: git init

  - tui:
      command: opencode
      steps:
        - activate: {role: menuitem, name: Settings}
        - fill: {label: Username, value: dmytri}
        - activate: {role: button, name: Save}
        - expect: {text: Saved}
        - capture: {role: message-pane, items: message, scope: all, as: conversation}

  - run: cargo test
```

Both forms parse to the same plan. Shorthand removes typing; it never adds capability and never weakens a default.

## Isolation

Omitting the `sandbox` block does not mean no sandbox. It means the secure defaults: an empty home, network denied, a controlled PATH, and read-only mounts. Grant what the target needs and nothing more.

- `mode: readonly` is the default for a mount.
- `mode: copy` gives the target a writable copy, leaving your fixture untouched.
- An environment variable reaches the program only when the plan names it.

On Linux the backend is Bubblewrap. If Bubblewrap is unavailable, Tinman fails loudly rather than silently running your agent unsandboxed.

## Semantic capture

`capture` collects the items of a scrolling pane into named structured data. The runtime scrolls, collects, deduplicates and returns. This is how you assert on a coding agent's whole conversation rather than the one screenful that happens to be visible.

```yaml
- capture: {role: message-pane, items: message, scope: all, as: conversation}
```

## Best practices

- Prefer role and name locators over key sequences. A recorded keystroke breaks when a menu gains an item; a locator does not.
- Run `tinman inspect` first to learn what a program actually exposes, then write locators against it.
- Grant the narrowest sandbox that lets the target work, and prefer `copy` over `writable` for fixtures.
- Keep recorded plans in version control and edit them by hand. They are meant to be read.
- Assert on captured structured data rather than on raw screen text where a pane scrolls.
- Never depend on inference in a test. If a plan needs a model to run, it is not a test.

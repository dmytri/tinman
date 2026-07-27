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

The TOM is the terminal's document object model: a tree of nested rectangles, following Ratatui's geometry of horizontal and vertical splits, with a semantic role attached to each region.

The roles are drawn from WAI-ARIA, so a role assistive technology understands is a role you already understand. Tinman coins none of its own. The full set:

`application`, `region`, `menu`, `menuitem`, `list`, `listitem`, `button`, `textbox`, `status`, `log`, `article`.

A role is added when a scenario needs it, not in anticipation, so this list is the whole of what a locator can address today.

Each region carries the presentation it was drawn with, the way a browser exposes a computed style beside the element tree:

- `style.foreground` and `style.background` are `default`, one of the sixteen named palette colours, a `{index}` into the 256-colour palette, or an `{red, green, blue}` triple.
- `style.bold` and `style.reverse` are booleans.
- A region whose cells are drawn differently carries no `style` at all, so an absent style means mixed rather than plain.

The model also carries `cursor`, a `{row, column}` where the terminal's cursor stands, absent when the program has hidden it. That is what tells a textbox being edited from a list being displayed.

It is a semantic reading of the rendered screen, not a reconstruction of the program that drew it.

Locators bind to the TOM by role and name, and resolution is mechanical. A test you write by hand needs no inference at all. Use `tinman inspect` to discover the roles, names and styles a program exposes:

```
tinman inspect opencode
```

## Commands

- `tinman record <command...>` captures a live session into an editable plan.
- `tinman test <plan>` runs a plan and reports whether it passed, replaying it exactly with no model invocation and no network.
- `tinman inspect <command...>` prints the terminal object model of a running program.
- `tinman driver` speaks the JSON driver protocol on stdin and stdout.
- `tinman man` writes Tinman's manual page as roff on stdout.
- `tinman completions <shell>` writes a completion script for that shell on stdout.

Both `man` and `completions` emit at runtime rather than shipping a generated file, so neither can drift from the parser and both work after `cargo install`:

```
tinman man
tinman completions bash
```

Running `tinman` with no arguments on a terminal opens the interactive assistant, which answers questions about Tinman and proposes commands for you to confirm. Redirect stdin or stdout and you get the conventional help instead.

## Driving Tinman from your own test suite

`tinman driver` exchanges one JSON-RPC 2.0 message per line on stdin and stdout, per the specification at https://www.jsonrpc.org/specification, so a test in pytest, jest, bun test or anything else needs only a subprocess and the JSON-RPC library your language already has.

```
tinman driver
```

Methods: `launch`, `activate`, `fill`, `press`, `expect`, `capture`, `screen`, `tom`, `sandbox`, `close`. Every request carries `jsonrpc`, `id` and `method`; every reply echoes the `id`. `launch` creates a session and takes no `session`; every other method names the session it addresses, because a driver holds several at once and a default would be a guess.

```json
{"jsonrpc": "2.0", "id": 1, "method": "launch", "params": {"command": "opencode"}}
{"jsonrpc": "2.0", "id": 2, "method": "activate", "params": {"session": "s1", "role": "menuitem", "name": "Settings"}}
{"jsonrpc": "2.0", "id": 3, "method": "fill", "params": {"session": "s1", "label": "Username", "value": "dmytri"}}
{"jsonrpc": "2.0", "id": 4, "method": "expect", "params": {"session": "s1", "text": "Saved"}}
```

A protocol fault is an `error` object carrying a reserved code. A failed expectation is not a protocol fault: the call succeeded and the product disagreed, so it returns a `result` whose `ok` is false, and your test framework reports the failure in its own voice.

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
        - capture: {role: log, items: article, scope: all, as: conversation}

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
tui: opencode
steps:
  - capture: {role: log, items: article, scope: all, as: conversation}
```

## Best practices

- Prefer role and name locators over key sequences. A recorded keystroke breaks when a menu gains an item; a locator does not.
- Run `tinman inspect` first to learn what a program actually exposes, then write locators against it.
- Grant the narrowest sandbox that lets the target work, and prefer `copy` over `writable` for fixtures.
- Keep recorded plans in version control and edit them by hand. They are meant to be read.
- Assert on captured structured data rather than on raw screen text where a pane scrolls.
- Never depend on inference in a test. If a plan needs a model to run, it is not a test.

Here you go:

# Build Clanker

Build a proof-of-concept for **Clanker**, a deterministic black-box testing framework for CLIs and full-screen TUIs.

## Premise

Clanker is to terminal applications what Webrat/Capybara/Selenium are to web applications.

It does **not** inspect application internals. It interacts only through an embedded PTY.

The key idea is:

* **Capture time:** inference is allowed.
* **Replay time:** absolutely deterministic. No model invocation.

The inference phase acts as a compiler that produces a mechanical test plan.

## Goals

Implement a working prototype demonstrating:

* launching a command inside an embedded PTY
* rendering the PTY inside a Ratatui application
* recording keyboard input
* parsing terminal output into a virtual screen
* generating a semantic Terminal Object Model (TOM)
* recording interaction sequences
* replaying them deterministically

Do **not** attempt to build a production-quality inference engine. Simple heuristics and placeholders are sufficient.

## Technology

Use Rust.

Preferred libraries:

* ratatui
* crossterm
* portable-pty
* vt100 (or equivalent ANSI parser)
* clap
* tokio
* serde
* serde_yaml

The project should compile into a single binary.

## Architecture

Separate the system into two phases.

### Capture

```
PTY
↓
ANSI parser
↓
screen buffer
↓
heuristic TOM inference
↓
record semantic actions
↓
generate YAML test plan
```

Inference may eventually be performed by an LLM.

Assume the future default will be OpenRouter using DeepSeek V4 Flash, but design the inference behind an interface.

Do not invoke any model during replay.

### Replay

```
load YAML
↓
spawn PTY
↓
parse screen
↓
execute deterministic navigation
↓
assertions
```

Replay must never require a network connection.

## Terminal Object Model

Treat the inferred TOM as the equivalent of the browser DOM.

It is **not** intended to reconstruct the original application.

It is simply a semantic representation of the rendered terminal.

The geometry may be modelled after Ratatui:

* nested rectangles
* horizontal/vertical splits

Augment it with semantic roles:

* menu
* menuitem
* list
* listitem
* table
* row
* column
* dialog
* button
* textbox
* statusbar
* message-pane
* message
* tree
* treeitem

The exact inference is not important for this prototype.

## YAML format

Design a constrained YAML schema.

Example:

```yaml
flow:
  - tui:
      command: opencode

      steps:
        - activate:
            role: menuitem
            name: Settings

        - fill:
            label: Username
            value: dmytri

        - activate:
            role: button
            name: Save

        - expect:
            text: Saved
```

This YAML is the canonical representation.

Do not invent a programming-language DSL.

## Higher-level flow

Support orchestration of multiple processes.

Example:

```yaml
flow:
  - run:
      command: git init

  - tui:
      command: lazygit
      steps:
        ...

  - run:
      command: cargo test

  - tui:
      command: opencode
      steps:
        ...
```

## Semantic capture

Support capture operations.

Example:

```yaml
- capture:
    role: message-pane
    items: message
    scope: all
    as: conversation
```

The runtime should mechanically:

* scroll
* collect items
* deduplicate
* return structured data

Inference determines how to perform this.

Replay performs it deterministically.

## Recording

The recorder should resemble Selenium IDE or Playwright Codegen.

The user interacts normally.

Clanker records:

* key presses
* screen transitions
* inferred semantic actions

Generate editable YAML.

## Initial command

Implement:

```
clanker record <command...>
```

The first milestone is:

* embed a PTY
* display it in Ratatui
* record keys
* capture screen snapshots
* emit a simple YAML interaction log

Keep the implementation clean, modular and heavily documented so additional inference engines and richer TOM semantics can be added later.


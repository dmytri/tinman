# Tinman

A deterministic black-box testing framework for CLIs and full-screen TUIs —
"webrat/capybara/selenium for terminals." Tinman drives real terminal programs,
including real coding agents, through an embedded PTY, and never inspects
application internals.

- **Capture time** may infer a mechanical test plan.
- **Replay time** is absolutely deterministic: no model invocation, no network.

Sandboxed execution is the default. On Linux, Tinman launches its target inside
a Bubblewrap sandbox; the operator's home, environment, and network are hidden
unless explicitly granted.

> **Status: early prototype.** Breaking changes are expected at 0.1.x.

## Try it

Discover the roles, names and styles a program puts on screen:

```
tinman inspect opencode
```

Capture a live session into an editable plan, then replay it with no model and
no network:

```
tinman record opencode
tinman test tinman.yaml
```

Drive Tinman from a test suite in any language over newline-delimited
JSON-RPC 2.0:

```
tinman driver
```

Ask it what to run, or read the manual:

```
tinman
tinman man
tinman completions bash
```

`cargo install` places the binary and nothing else, so nothing lands in your
manual path. Redirect `tinman man` into `/usr/local/share/man/man1/tinman.1`
and run `mandb`, and `man tinman` works from then on. Emitting the page on
demand is what keeps it from drifting away from the parser that enforces it.

## The Terminal Object Model

Tinman reads the rendered screen as a tree of nested regions carrying WAI-ARIA
roles, accessible names, and the presentation they were drawn with. Locators
bind to that model by role and name rather than to cell coordinates, so a test
survives a layout change that would break a recorded keystroke.

The full vocabulary is in the bundled skill at `assets/skill/SKILL.md`, which is
also what a coding agent reads to learn Tinman.

## License

Licensed under the BSD Zero Clause License (0BSD). See `LICENSE`.

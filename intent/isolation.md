> **Historical.** A dated intent source from Tinman's design, kept for provenance
> because scantlings cite it as where an invariant came from. It is not a
> requirement, and it does not describe current behaviour: parts of it name
> commands, options and fields that no longer exist. Binding behaviour lives in
> `features/`, mechanical shape in `scantlings/`, tooling values in `RIGGING.md`.

Yes—implement the abstraction now, but only implement the Bubblewrap backend.

# Addendum: Sandbox Abstraction with Bubblewrap First

Isolation is a core part of Clanker.

The initial implementation must support **Bubblewrap on Linux**, while defining a clean backend abstraction so macOS support can be added soon without changing the harness format or PTY architecture.

Do not implement the macOS backend yet.

## Architecture

Use this execution model:

```text
Harness YAML
    ↓
Sandbox specification
    ↓
Sandbox backend
    └── BubblewrapBackend
    ↓
Prepared process
    ↓
PTY
    ↓
Agent
```

The PTY layer must not know about Bubblewrap directly.

The sandbox backend should prepare the command, environment, filesystem layout and working directory that the PTY runner launches.

## Sandbox interface

Define a small Rust trait or equivalent abstraction.

Conceptually:

```rust
trait SandboxBackend {
    fn prepare(
        &self,
        spec: &SandboxSpec,
        command: &CommandSpec,
    ) -> Result<PreparedProcess>;
}
```

`PreparedProcess` should contain everything required to launch the process:

```rust
struct PreparedProcess {
    program: PathBuf,
    args: Vec<OsString>,
    env: Vec<(OsString, OsString)>,
    cwd: Option<PathBuf>,
    cleanup: Vec<CleanupResource>,
}
```

The PTY runner accepts only `PreparedProcess`.

It must not construct Bubblewrap arguments itself.

Keep the interface narrow. Do not create a large plugin system or prematurely model platform-specific concepts.

## Initial backends

Define these backend identities:

```rust
enum SandboxKind {
    Bubblewrap,
    Mac,
    None,
}
```

Implement only:

```rust
BubblewrapBackend
```

The `Mac` variant may return a clear unsupported-backend error for now.

`None` may exist only for development and debugging, and must require an explicit unsafe option.

For normal Linux runs, Bubblewrap is the default.

## Harness specification

The public YAML format must be backend-neutral.

Do not expose Bubblewrap flags in the canonical harness format.

Example:

```yaml
sandbox:
  backend: auto
  home: empty
  network: deny

  env:
    OPENAI_API_KEY:
      from: fixture

  path:
    - ./fixtures/bin
    - /usr/bin

  mounts:
    - source: ./fixtures/project
      target: /workspace
      mode: copy

    - source: ./fixtures/opencode-config
      target: ~/.config/opencode
      mode: readonly

    - source: ./fixtures/skills
      target: ~/.config/opencode/skills
      mode: readonly

    - source: ./fixtures/plugins
      target: ~/.local/share/opencode/plugins
      mode: readonly
```

Use portable concepts such as:

* empty or fixture-backed home
* clean environment
* controlled `PATH`
* read-only mount
* writable mount
* copied fixture
* isolated working directory
* network allow or deny
* temporary directories
* injected secrets or credentials

Avoid concepts that only make sense in Bubblewrap.

## Backend selection

Support:

```yaml
sandbox:
  backend: auto
```

For the initial release:

* `auto` selects Bubblewrap on Linux
* `bubblewrap` explicitly selects Bubblewrap
* `mac` returns “not implemented”
* `none` requires an explicit unsafe CLI flag

Fail clearly when Bubblewrap is unavailable.

Do not silently fall back to an unsandboxed process.

## Bubblewrap backend

Translate the portable sandbox specification into Bubblewrap arguments.

The Bubblewrap backend should provide:

* isolated mount namespace
* isolated process namespace where practical
* optional network namespace
* temporary HOME
* temporary XDG directories
* controlled environment
* controlled `PATH`
* minimal writable directories
* read-only system mounts
* explicitly mounted fixtures
* explicitly selected working directory

Prefer read-only mounts by default.

The real user home, agent configuration, credentials, skills, plugins and local binaries must not be visible unless explicitly mounted.

## macOS readiness

Structure the portable specification so a future `MacSandboxBackend` can implement the same semantics using a combination of:

* temporary HOME and XDG directories
* controlled environment
* copied or linked fixture trees
* restricted `PATH`
* native sandboxing where viable
* a lightweight VM backend if stronger isolation is needed

Do not force the future macOS implementation to emulate raw Linux mount flags.

The abstraction should express desired environment semantics, not Bubblewrap mechanics.

## Resource lifecycle

A backend may create temporary directories, copied fixtures or generated configuration.

Represent their lifecycle explicitly.

Cleanup must happen:

* after successful completion
* after test failure
* after PTY launch failure
* after cancellation where possible

Use RAII-style cleanup rather than scattered manual deletion.

## Security behaviour

Clanker is intended to run coding agents, which may execute arbitrary commands.

Therefore:

* sandboxed execution is the default
* unavailable isolation is a hard failure
* unsandboxed mode is visibly marked unsafe
* the operator's real HOME must never be inherited by default
* the operator's full environment must never be inherited by default
* the host `PATH` must never be inherited implicitly
* secrets must only be injected explicitly

## Scope

Implement now:

* backend-neutral `SandboxSpec`
* `SandboxBackend` abstraction
* `PreparedProcess`
* backend selection
* Bubblewrap backend
* explicit unsupported macOS backend
* explicit unsafe local backend
* integration with the PTY runner
* tests for Bubblewrap argument generation and environment isolation

Do not implement now:

* functional macOS isolation
* containers
* Docker or Podman backends
* remote execution
* backend plugins
* Windows support

## Design principle

Build the abstraction now because macOS support is expected soon.

Build only the Bubblewrap implementation now because it is the current execution target.

The public harness format and PTY runner must remain unchanged when the macOS backend is added.



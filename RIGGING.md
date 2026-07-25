# Rigging

## Stack
- language: Rust
- runtime: rustc stable, edition 2024
- packageManager: cargo

## Directories
- implementation: src
- specs: features
- verification: tests
- assets: none
- scantlings: scantlings

## Commands
- discover: none
- focused: `ref="{scenario}"; cargo test --test cucumber -- -i "${ref%%:*}" --name "^${ref#*:}$"`
- broad: `CUCUMBER_FILTER_TAGS="not @sandbox and not @captain and not @shipwright" cargo test --test cucumber`
- broad-sandbox: `CUCUMBER_FILTER_TAGS="@sandbox and not @captain and not @shipwright" cargo test --test cucumber`
- coverage: none
- step-usage: none
- plank-inventory: none
- typecheck: `cargo check --all-targets`
- lint: `cargo fmt --check && cargo clippy --all-targets -- -D warnings`
- conformance: none

## Perturbation
- message: `PERTURBATION: consider current durable context; remove when fixed`
- perturb: `panic!("PERTURBATION: consider current durable context; remove when fixed");`

## Tiers
- default: @logic
- sandbox: @sandbox
- policy: The default tier holds pure, local, deterministic tests that need no external tool; untagged scenarios belong to it. The @sandbox tier holds scenarios that launch a real process under Bubblewrap and requires the `bwrap` binary and unprivileged user namespaces.
- weather: none
- runrecord: none

## Dependencies
- policy: locked
- dependency: cucumber
- dependency: tokio
- dependency: portable-pty
- dependency: vt100
- dependency: ratatui
- dependency: crossterm
- dependency: clap
- dependency: serde
- dependency: serde_yaml
- dependency: serde_json
- dependency: jsonschema

## Outbound
- outbound: none

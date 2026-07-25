# Rigging

## Stack
- language: Rust
- runtime: rustc stable, edition 2024
- packageManager: cargo

## Directories
- implementation: src
- specs: features
- verification: tests
- assets: assets
- scantlings: scantlings
- scantlings: scantlings/verification-conformance

## Commands
- discover: none
- focused: `ref="{scenario}"; cargo test --test cucumber -- -i "${ref%%:*}" --name "^${ref#*:}$"`
- broad: `CUCUMBER_FILTER_TAGS="not @sandbox and not @inference and not @captain and not @shipwright" cargo test --test cucumber`
- broad-sandbox: `CUCUMBER_FILTER_TAGS="@sandbox and not @captain and not @shipwright" cargo test --test cucumber`
- broad-inference: `CUCUMBER_FILTER_TAGS="@inference and not @captain and not @shipwright" cargo test --test cucumber`
- coverage: `CUCUMBER_FILTER_TAGS="not @sandbox and not @inference and not @captain and not @shipwright" cargo llvm-cov --test cucumber --summary-only`
- coverage-sandbox: `CUCUMBER_FILTER_TAGS="@sandbox and not @captain and not @shipwright" cargo llvm-cov --test cucumber --summary-only`
- coverage-inference: `CUCUMBER_FILTER_TAGS="@inference and not @captain and not @shipwright" cargo llvm-cov --test cucumber --summary-only`
- step-usage: none
- plank-inventory: `ast-grep scan --inline-rules '{id: planks, language: rust, rule: {kind: line_comment, regex: "@planks"}}' --json=compact src`
- typecheck: `cargo check --all-targets`
- lint: `cargo fmt --check && cargo clippy --all-targets -- -D warnings`
- conformance: `ast-grep scan`

## Perturbation
- message: `PERTURBATION: consider current durable context; remove when fixed`
- perturb: `panic!("PERTURBATION: consider current durable context; remove when fixed");`

## Tiers
- default: @logic
- sandbox: @sandbox
- inference: @inference
- policy: The default tier holds pure, local, deterministic tests that need no external tool; untagged scenarios belong to it. The @sandbox tier holds scenarios that launch a real process under Bubblewrap and requires the `bwrap` binary and unprivileged user namespaces. The @inference tier holds scenarios that call the configured inference provider for real and requires `TINMAN_API_KEY`, read from the environment or from a git-ignored `.env` file, with optional `TINMAN_BASE_URL` and `TINMAN_MODEL` overrides defaulting to OpenRouter and deepseek/deepseek-v4-flash; it costs money per run and is never on the inner loop.
- weather: target/tinman-weather.jsonl
- runrecord: target/tinman-runrecord.jsonl

## Dependencies
- policy: locked
- dependency: cucumber
- dependency: tokio
- dependency: portable-pty
- dependency: alacritty_terminal
- dependency: ratatui
- dependency: crossterm
- dependency: clap
- dependency: serde
- dependency: serde_yaml
- dependency: serde_json
- dependency: jsonschema
- dependency: ureq
- dependency: dotenvy
- dependency: cargo-llvm-cov
- dependency: ast-grep

## Outbound
- outbound: crates.io
- ship: `cargo publish`
- verify: `cargo search tinman`

# Repository Guidelines

## Project Structure & Module Organization

This is a small Rust 2024 crate for bridging MQTT JSON messages to CoAP POST requests. Reusable startup/configuration code lives in `src/lib.rs` and `src/startup.rs`. The runnable binary is `src/bin/mqtt2coap.rs`, which owns MQTT subscription handling, JSON value conversion, and CoAP sending. `build.rs` captures build metadata, `Cargo.toml` defines dependencies, and `rustfmt.toml` contains formatting policy. `install.sh` and `build` support deployment and cross-build workflows. There is no dedicated `tests/` directory or asset tree.

## Build, Test, and Development Commands

- `cargo build`: compile the debug build for local iteration.
- `cargo build --release`: produce the optimized release binary.
- `cargo run --bin mqtt2coap -- --mqtt-host localhost --topics test123`: run the bridge locally with explicit options.
- `cargo test`: run tests when present.
- `cargo fmt`: apply the repository Rust formatting rules.
- `cargo clippy --all-targets --all-features`: run lint checks across binaries and tests.
- `./build pc build`: build the x86_64 debug target via the helper script.
- `./build pc release`: build the x86_64 release target via the helper script.
- `./build raspi release`: cross-compile for Raspberry Pi ARMv7; requires the configured ARM linker/toolchain.

## Coding Style & Naming Conventions

Use standard Rust naming: `snake_case` for functions, modules, and variables; `PascalCase` for types; `SCREAMING_SNAKE_CASE` for constants. Add startup or option code under `src/startup.rs` only when shared beyond the binary. Format with `cargo fmt`; the project uses `max_width = 120`, crate-level import grouping, and grouped standard/external/crate imports.

## Testing Guidelines

Add focused unit tests next to the code they exercise using `#[cfg(test)] mod tests`, especially for JSON conversion and topic rewriting behavior. Use integration tests under `tests/` only for end-to-end CLI or network-facing behavior. Avoid requiring live MQTT or CoAP services in default tests. Always run `cargo test` before submitting changes.

## Commit & Pull Request Guidelines

Recent commits use short, direct messages such as `cargo update`; keep messages concise and imperative, for example `add topic conversion tests`. Pull requests should describe the behavioral change, list verification commands, mention any MQTT/CoAP compatibility impact, and link related issues when available. Include sample command lines or logs when changing CLI or network behavior.

## Security & Configuration Tips

Do not commit broker credentials, private hostnames, or production endpoint details. Pass runtime settings through CLI flags such as `--mqtt-host`, `--topics`, `--topic-prefix`, and `--coap-url`. Treat malformed JSON and network failures as expected runtime inputs, and avoid logging sensitive payload data.

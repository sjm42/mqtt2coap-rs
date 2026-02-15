# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

mqtt2coap-rs is a Rust bridge that forwards MQTT messages to CoAP endpoints. It subscribes to MQTT topics, parses JSON
payloads, converts each field to a float value, and POSTs each key-value pair to a CoAP URL.

## Build Commands

```bash
cargo build                    # Debug build
cargo build --release          # Release build (opt-level 3, LTO fat, codegen-units 1)
cargo clippy                   # Lint
cargo fmt                      # Format (max_width=120, see rustfmt.toml)
./build pc build               # Cross-compile wrapper for x86_64
./build raspi release           # Cross-compile for armv7 (Raspberry Pi)
```

There are no tests in this project.

## Architecture

The entire application is ~177 lines across three files:

- **`src/bin/mqtt2coap.rs`** — Main binary. Connects to MQTT broker via `rumqttc`, subscribes to topics, runs an async
  event loop (Tokio). Each incoming `Publish` message spawns a task that calls `handle_msg` → `coap_send`.
- **`src/startup.rs`** — CLI argument parsing (`clap` derive) in `OptsCommon`. Configures logging via `tracing`/
  `tracing-subscriber` with `--verbose`/`--debug`/`--trace` flags. Prints build metadata on startup.
- **`src/lib.rs`** — Re-exports from startup module.

### Message flow

```
MQTT Publish (JSON) → parse JSON object → for each key-value pair:
  - convert value to f64 (numbers direct, bools 0/1, strings "on"/"1"/"true" → 1.0 else 0.0)
  - strip topic prefix if configured
  - POST "{topic}/{key} {value:.2}" to CoAP URL
```

### Key dependencies

- `rumqttc` — async MQTT client
- `coap` (UdpCoAPClient) — CoAP POST requests
- `tokio` — async runtime
- `clap` — CLI parsing
- `anyhow` — error handling
- `serde_json` — JSON parsing
- `tracing` / `tracing-subscriber` — structured logging

### Build script

`build.rs` uses the `build-data` crate to embed git branch, commit, source timestamp, and rustc version into the binary.

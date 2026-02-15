# mqtt2coap-rs

An async Rust bridge that subscribes to MQTT topics, parses JSON payloads, and forwards each field as a separate CoAP POST request. Built for IoT sensor data pipelines where devices publish JSON over MQTT and downstream systems consume CoAP.

## How it works

Given an MQTT message on topic `sensors/room1`:

```json
{"temperature": 22.5, "humidity": 60, "heater": true}
```

The bridge sends three CoAP POST requests to the configured endpoint:

```
POST coap://localhost/store_data  ←  "room1/temperature 22.50"
POST coap://localhost/store_data  ←  "room1/humidity 60.00"
POST coap://localhost/store_data  ←  "room1/heater 1.00"
```

### Value conversion

All JSON values are converted to `f64` and formatted to two decimal places:

| JSON type | Conversion |
|-----------|------------|
| Number    | Used directly |
| Boolean   | `true` → `1.00`, `false` → `0.00` |
| String    | `"on"`, `"1"`, `"true"` → `1.00`, everything else → `0.00` |

The first path segment of the MQTT topic is stripped (treated as a prefix), so `sensors/room1` becomes `room1`.

## Building

Requires stable Rust toolchain.

```bash
cargo build --release
```

### Cross-compilation

A helper script supports x86_64 and ARMv7 (Raspberry Pi) targets:

```bash
./build pc release       # x86_64-unknown-linux-gnu
./build raspi release    # armv7-unknown-linux-gnueabihf
```

ARM targets require the appropriate cross-linker (configured in `.cargo/config.toml`).

## Usage

```
mqtt2coap [OPTIONS]
```

| Option | Default | Description |
|--------|---------|-------------|
| `--mqtt-host` | `localhost` | MQTT broker hostname |
| `--mqtt-port` | `1883` | MQTT broker port |
| `--topics` | `test123` | Comma-separated list of topics to subscribe to |
| `--topic-prefix` | *(empty)* | Prefix prepended to each topic when subscribing |
| `--coap-url` | `coap://localhost/store_data` | CoAP endpoint URL for POST requests |
| `-v`, `--verbose` | | Enable info-level logging |
| `-d`, `--debug` | | Enable debug-level logging |
| `-t`, `--trace` | | Enable trace-level logging |

### Example

Subscribe to `zigbee2mqtt/sensors/temperature` and `zigbee2mqtt/sensors/humidity`, forwarding to a CoAP server:

```bash
mqtt2coap \
  --mqtt-host 192.168.1.10 \
  --topic-prefix "zigbee2mqtt/" \
  --topics "sensors/temperature,sensors/humidity" \
  --coap-url "coap://192.168.1.20/store_data" \
  --verbose
```

## License

MIT
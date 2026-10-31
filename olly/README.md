# olly

[![Crates.io](https://img.shields.io/crates/v/olly)](https://crates.io/crates/olly)
[![Documentation](https://docs.rs/uhoh/badge.svg)][docs]

`olly` is an observability preset library for Rust applications.
Observability means understanding a running application through its outputs.
A preset supplies a ready-made configuration.

The current preset sets up logging with `tracing`.
An event is a record of something that happens.
`olly` writes events to standard output, the normal output stream.
By default, it uses JSON and includes events at `info`, `warn`, and `error` levels.

Future plans include OpenTelemetry support and metrics, which are numeric measurements of application behavior.

## Get started

This example uses a local copy of the `bitkit-rs` repository.
Add these dependencies to the `Cargo.toml` file for your application:

```toml
[dependencies]
olly = { path = "../bitkit-rs/olly" }
tracing = "0.1"
```

Adjust the `olly` path to match your local repository location.
In `main()`, call `olly::init_tracing()` at application startup, before the first event:

```rust
olly::init_tracing();
tracing::info!(service = "worker", "Application started");
```

## Logging configuration

Set environment variables before you start the application.
`olly` reads `LOG_FORMAT` and `RUST_LOG` when you call `init_tracing()`.
Later changes to these variables do not update the active logging configuration.

`LOG_FORMAT` selects the output format:

- `json` writes each event as a JSON object on one line.
- `pretty` writes each event as text across multiple lines.
- Any other value selects the default text format from `tracing-subscriber` and emits a warning event.

If `LOG_FORMAT` is absent or contains invalid Unicode, `olly` uses `json`.
The values are case-sensitive.
An empty value also selects the default text format and emits a warning event.
The active logging configuration determines whether the warning appears.

A filter selects which events appear in the output.
`RUST_LOG` sets the filter with the syntax from `tracing-subscriber::EnvFilter`.
If `RUST_LOG` is absent, contains invalid Unicode, or contains invalid filter syntax, `olly` uses `info`.
A valid empty filter disables all events.

From the application directory, run this command for text output with debug events:

```sh
LOG_FORMAT=pretty RUST_LOG=debug cargo run
```

A target identifies the source of an event.
By default, `tracing` uses the Rust module path as the target.
For debug events from `my_app` and `info` events elsewhere, use this command:

```sh
LOG_FORMAT=json RUST_LOG=info,my_app=debug cargo run
```

Replace `my_app` with the target that you want to inspect.
`RUST_LOG=off` disables all events, including the warning for an unknown `LOG_FORMAT` when the new configuration takes effect.

## Application setup

A subscriber receives and processes events from `tracing`.
`init_tracing()` tries to install a subscriber as the default for the whole process.
Call it once from the application entry point.
Libraries must leave this choice to the application that uses them.

The function ignores setup errors and returns `()`.
If a global subscriber already exists, `olly` leaves it in place.
Repeated calls do not update its format or filter.
Use `tracing-subscriber` directly if your application needs custom output destinations or access to setup errors.

## Local documentation

From the `bitkit-rs` repository root, build the Rust documentation:

```sh
mise run docs olly
```

Open `target/doc/olly/index.html` in a browser.
The documentation includes the README and the reference for `init_tracing()`.
To run the tests and documentation examples, use this command from the same directory:

```sh
cargo test --package olly --locked
```

[docs]: https://docs.rs/olly

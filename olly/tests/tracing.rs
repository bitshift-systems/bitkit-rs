use std::process::Command;

#[test]
fn tracing_probe() {
    if std::env::var_os("TACTICA_TRACING_PROBE").is_some() {
        olly::init_tracing();
        tracing::debug!(target: "startup_probe", "startup debug marker");
        tracing::info!(target: "startup_probe", "startup info marker");
    }
}

#[test]
fn logging_respects_rust_log() {
    for format in ["pretty", "json"] {
        for (filter, debug_expected) in [(Some("startup_probe=debug"), true), (None, false)] {
            let mut command = Command::new(std::env::current_exe().unwrap());
            command
                .args(["--exact", "tracing_probe", "--nocapture"])
                .env("TACTICA_TRACING_PROBE", "1")
                .env("LOG_FORMAT", format)
                .env_remove("RUST_LOG");
            if let Some(filter) = filter {
                command.env("RUST_LOG", filter);
            }
            let output = command.output().unwrap();
            assert!(output.status.success());
            let stdout = String::from_utf8(output.stdout).unwrap();
            assert!(stdout.contains("startup info marker"), "{format}: {stdout}");
            assert_eq!(
                stdout.contains("startup debug marker"),
                debug_expected,
                "{format}, filter {filter:?}: {stdout}",
            );
        }
    }
}

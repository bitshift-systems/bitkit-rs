#![doc = include_str!("../README.md")]

/// Sets up logging from the application environment.
///
/// A subscriber receives and processes events from `tracing`.
/// This function tries to install a subscriber as the default for the whole
/// process. It writes events to standard output, the normal output stream.
/// Call this function once at application startup, before the first event.
///
/// # Configuration
///
/// `LOG_FORMAT=json` selects JSON output, with one event per line.
/// `LOG_FORMAT=pretty` selects text output across multiple lines.
/// If `LOG_FORMAT` is absent or contains invalid Unicode, this function uses JSON.
/// Any other value, including an empty string, selects the default text format
/// from `tracing-subscriber` and emits a warning event.
/// The values are case-sensitive.
/// The active logging configuration determines whether the warning appears.
///
/// A filter selects which events appear in the output.
/// `RUST_LOG` sets the filter with the syntax from
/// [`EnvFilter`](tracing_subscriber::EnvFilter).
/// If `RUST_LOG` is absent, contains invalid Unicode, or contains invalid filter
/// syntax, this function uses `info`.
/// This default includes events at `info`, `warn`, and `error` levels.
/// A valid empty filter disables all events.
///
/// # Setup errors
///
/// This function ignores setup errors and returns `()`.
/// If a global subscriber already exists, this function leaves it in place.
/// Repeated calls do not update its format or filter.
///
/// # Example
///
/// Call this function from the application entry point.
/// Use `tracing` to record events after setup.
/// With the default configuration, this example writes an event in JSON:
///
/// ```
/// olly::init_tracing();
/// tracing::info!(service = "worker", "Application started");
/// ```
pub fn init_tracing() {
    let format = std::env::var("LOG_FORMAT").unwrap_or_else(|_| "json".to_string());

    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    if format == "json" {
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .json()
            .try_init()
            .ok();
    } else if format == "pretty" {
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .pretty()
            .try_init()
            .ok();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .try_init()
            .ok();
        tracing::warn!(
            "Unknown LOG_FORMAT '{}', falling back to default format",
            format
        );
    }
}

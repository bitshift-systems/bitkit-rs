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

#[derive(Debug)]
pub enum TraceInitError {
    InvalidFilter(tracing_subscriber::filter::ParseError),
    InstallSubscriber(tracing::dispatcher::SetGlobalDefaultError),
}

impl std::fmt::Display for TraceInitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidFilter(e) => write!(f, "invalid telemetry filter: {e}"),
            Self::InstallSubscriber(e) => write!(f, "failed to install telemetry subscriber: {e}"),
        }
    }
}

impl std::error::Error for TraceInitError {}

#[cfg(test)]
mod tests {
    use configuration::RateLimitConfig;

    use super::*;

    #[test]
    fn displays_invalid_filter_variant() {
        let parse_err = tracing_subscriber::EnvFilter::try_new("!!!bad").unwrap_err();
        let err = TraceInitError::InvalidFilter(parse_err);
        assert!(err.to_string().contains("invalid telemetry filter"));
    }

    #[test]
    fn displays_install_subscriber_variant() {
        // init_tracing twice in the same process: second call fails.
        let cfg = std::sync::Arc::new(configuration::Config {
            primary: configuration::PrimaryConfig {
                env: "test".into(),
                name: "test".into(),
            },
            server: configuration::ServerConfig {
                host: "127.0.0.1".into(),
                port: 0,
                read_timeout_secs: 1,
                write_timeout_secs: 1,
                idle_timeout_secs: 1,
                request_timeout_secs: 1,
                shutdown_timeout_secs: 1,
                min_graceful_shutdown_secs: 0,
                max_body_size_bytes: 1024,
                cors_allowed_origins: vec![],
                path_prefix: "/".into(),
                rate_limit: RateLimitConfig::default(),
            },
            mqtt: configuration::MqttConfig {
                broker_uri: "mqtt://localhost:1883".into(),
                username: None,
                password: None,
                client_id: "test1".to_string(),
                keep_alive_secs: 6000000,
                reconnect_delay_secs: 500,
                capacity: 15,
            },
            telemetry: configuration::TelemetryConfig {
                service_name: "test".into(),
                environment: "test".into(),
                filter: "info".into(),
                format: configuration::LogFormat::Compact,
                ansi: false,
                include_file: false,
                include_line_number: false,
            },
            store: None,
        });
        let _ = crate::init_tracing(cfg.clone());
        let result = crate::init_tracing(cfg);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("failed to install telemetry subscriber"));
    }

    #[test]
    fn implements_error_trait() {
        let err: Box<dyn std::error::Error> = Box::new(TraceInitError::InvalidFilter(
            tracing_subscriber::EnvFilter::try_new("!!!bad").unwrap_err(),
        ));
        assert!(!err.to_string().is_empty());
    }
}

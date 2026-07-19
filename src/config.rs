use std::{env, time::Duration};

use secrecy::SecretString;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub database_url: SecretString,
    pub jwt_secret: SecretString,
    pub ping_interval_secs: u64,
    pub num_ping_workers: usize,
    pub observability: ObservabilityConfig,
}

#[derive(Debug, Clone)]
pub struct ObservabilityConfig {
    pub service_name: String,
    pub service_version: String,
    pub deployment_environment: String,
    pub otlp_endpoint: Option<String>,
    pub slow_query_threshold: Duration,
}

impl Config {
    pub fn from_env() -> Self {
        let port: u16 = env::var("PORT")
            .expect("PORT not set in environment variables")
            .parse()
            .expect("PORT must be a valid u16");

        let database_url = SecretString::from(
            env::var("DATABASE_URL").expect("DATABASE_URL must be set in environment variables"),
        );

        let jwt_secret = SecretString::from(
            env::var("JWT_SECRET").expect("JWT_SECRET must be set in environment variables"),
        );

        let ping_interval_secs = env::var("PING_INTERVAL_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(60);

        let num_ping_workers = env::var("NUM_PING_WORKERS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(5);

        let slow_query_threshold_ms = env::var("SLOW_QUERY_THRESHOLD_MS")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(250);

        let otlp_endpoint = env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
            .ok()
            .filter(|endpoint| !endpoint.trim().is_empty());

        Self {
            port,
            database_url,
            jwt_secret,
            ping_interval_secs,
            num_ping_workers,
            observability: ObservabilityConfig {
                service_name: env::var("OTEL_SERVICE_NAME")
                    .unwrap_or_else(|_| env!("CARGO_PKG_NAME").to_string()),
                service_version: env::var("OTEL_SERVICE_VERSION")
                    .unwrap_or_else(|_| env!("CARGO_PKG_VERSION").to_string()),
                deployment_environment: env::var("DEPLOYMENT_ENVIRONMENT")
                    .unwrap_or_else(|_| "local".to_string()),
                otlp_endpoint,
                slow_query_threshold: Duration::from_millis(slow_query_threshold_ms),
            },
        }
    }
}

use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub jwt_secret: String,
    pub ping_interval_secs: u64,
    pub num_ping_workers: usize,
}

impl Config {
    pub fn from_env() -> Self {
        let port: u16 = env::var("PORT")
            .expect("PORT not set in environment variables")
            .parse()
            .expect("PORT must be a valid u16");

        let database_url =
            env::var("DATABASE_URL").expect("DATABASE_URL must be set in environment variables");

        let jwt_secret =
            env::var("JWT_SECRET").expect("JWT_SECRET must be set in environment variables");

        let ping_interval_secs = env::var("PING_INTERVAL_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(60);

        let num_ping_workers = env::var("NUM_PING_WORKERS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(5);

        Self {
            port,
            database_url,
            jwt_secret,
            ping_interval_secs,
            num_ping_workers,
        }
    }
}

//! Configuration

use std::env;

#[allow(dead_code)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub platform_domain: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5433/saas_platform".to_string()),
            port: env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3000),
            platform_domain: env::var("PLATFORM_DOMAIN")
                .unwrap_or_else(|_| "saas.local".to_string()),
        }
    }
}

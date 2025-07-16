use serde::Deserialize;
use std::env;

pub const PAYMENTS_QUEUE_KEY: &str = "payments_queue";
pub const PROCESSED_PAYMENTS_SET_KEY: &str = "processed_payments";
pub const DEFAULT_PROCESSOR_HEALTH_KEY: &str = "health:default";
pub const FALLBACK_PROCESSOR_HEALTH_KEY: &str = "health:fallback";

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
	pub redis_url: String,
	pub default_payment_processor_url: String,
	pub fallback_payment_processor_url: String,
	pub server_keepalive: u64,
}

impl Config {
	pub fn load() -> Result<Self, config::ConfigError> {
		let config_builder = config::Config::builder()
			.add_source(config::Environment::with_prefix("APP"))
			.build()?;

		config_builder.try_deserialize()
	}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_load() {
        // Set environment variables for testing
        env::set_var("APP_REDIS_URL", "redis://test_redis/");
        env::set_var("APP_DEFAULT_PAYMENT_PROCESSOR_URL", "http://test_default/");
        env::set_var("APP_FALLBACK_PAYMENT_PROCESSOR_URL", "http://test_fallback/");
        env::set_var("APP_SERVER_KEEPALIVE", "120");

        let config = Config::load().expect("Failed to load config in test");

        assert_eq!(config.redis_url, "redis://test_redis/");
        assert_eq!(config.default_payment_processor_url, "http://test_default/");
        assert_eq!(config.fallback_payment_processor_url, "http://test_fallback/");
        assert_eq!(config.server_keepalive, 120);

        // Clean up environment variables
        env::remove_var("APP_REDIS_URL");
        env::remove_var("APP_DEFAULT_PAYMENT_PROCESSOR_URL");
        env::remove_var("APP_FALLBACK_PAYMENT_PROCESSOR_URL");
        env::remove_var("APP_SERVER_KEEPALIVE");
    }
}
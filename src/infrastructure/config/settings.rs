use std::borrow::Cow;

use config::Environment;
use serde::Deserialize;

use crate::domain::payment_processor::PaymentProcessorKey;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
	pub redis_url: Cow<'static, str>,
	pub default_payment_processor_url: Cow<'static, str>,
	pub fallback_payment_processor_url: Cow<'static, str>,
	pub server_keepalive: u64,
	pub report_url: Option<Cow<'static, str>>,
}

impl Config {
	pub fn load() -> Result<Self, config::ConfigError> {
		Self::load_from(Environment::with_prefix("APP"))
	}

	fn load_from(environment: Environment) -> Result<Self, config::ConfigError> {
		let config_builder =
			config::Config::builder().add_source(environment).build()?;

		config_builder.try_deserialize()
	}

	pub fn get_processors_keys(&self) -> Vec<PaymentProcessorKey> {
		vec![
			PaymentProcessorKey::new(
				"default",
				self.default_payment_processor_url.clone(),
			),
			PaymentProcessorKey::new(
				"fallback",
				self.fallback_payment_processor_url.clone(),
			),
		]
	}
}

#[cfg(test)]
mod tests {
	use std::collections::HashMap;

	use super::*;

	const APP_PREFIX: &str = "APP";

	#[test]
	fn test_config_load_fails_when_app_configs_are_unavailable() {
		assert!(Config::load().is_err());
	}

	#[test]
	fn test_config_load_app_settings() {
		let source = Environment::with_prefix(APP_PREFIX).source(Some({
			let mut env = HashMap::new();
			env.insert("APP_REDIS_URL".into(), "redis://test_redis/".into());
			env.insert(
				"APP_DEFAULT_PAYMENT_PROCESSOR_URL".into(),
				"http://test_default/".into(),
			);
			env.insert(
				"APP_FALLBACK_PAYMENT_PROCESSOR_URL".into(),
				"http://test_fallback/".into(),
			);
			env.insert("APP_SERVER_KEEPALIVE".into(), "120".into());
			env.insert("APP_REPORT_URL".into(), "/tmp/reports".into());
			env
		}));

		let config =
			Config::load_from(source).expect("Failed to load config in test");

		assert_eq!(config.redis_url, "redis://test_redis/");
		assert_eq!(config.default_payment_processor_url, "http://test_default/");
		assert_eq!(
			config.fallback_payment_processor_url,
			"http://test_fallback/"
		);
		assert_eq!(config.server_keepalive, 120);
		assert_eq!(config.report_url, Some(Cow::from("/tmp/reports")));
	}

	#[test]
	fn test_config_load_without_report_url() {
		let source = Environment::with_prefix(APP_PREFIX).source(Some({
			let mut env = HashMap::new();
			env.insert(
				"APP_REDIS_URL".into(),
				"redis://test_redis_no_report/".into(),
			);
			env.insert(
				"APP_DEFAULT_PAYMENT_PROCESSOR_URL".into(),
				"http://test_default_no_report/".into(),
			);
			env.insert(
				"APP_FALLBACK_PAYMENT_PROCESSOR_URL".into(),
				"http://test_fallback_no_report/".into(),
			);
			env.insert("APP_SERVER_KEEPALIVE".into(), "120".into());
			env
		}));

		let config =
			Config::load_from(source).expect("Failed to load config in test");

		assert_eq!(config.redis_url, "redis://test_redis_no_report/");
		assert_eq!(
			config.default_payment_processor_url,
			"http://test_default_no_report/"
		);
		assert_eq!(
			config.fallback_payment_processor_url,
			"http://test_fallback_no_report/"
		);
		assert_eq!(config.server_keepalive, 120);
		assert_eq!(config.report_url, None);
	}
}

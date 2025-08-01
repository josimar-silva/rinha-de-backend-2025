use std::borrow::Cow;
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use circuitbreaker_rs::{CircuitBreaker, DefaultPolicy};

use crate::domain::health_status::HealthStatus;
use crate::domain::payment_processor::{PaymentProcessor, PaymentProcessorKey};
use crate::domain::payment_router::PaymentRouter;
use crate::use_cases::process_payment::PaymentProcessingError;

#[derive(Clone)]
pub struct InMemoryPaymentRouter {
	pub default_processor:  Arc<RwLock<PaymentProcessor>>,
	pub fallback_processor: Arc<RwLock<PaymentProcessor>>,
	pub default_breaker:    CircuitBreaker<DefaultPolicy, PaymentProcessingError>,
	pub fallback_breaker:   CircuitBreaker<DefaultPolicy, PaymentProcessingError>,
}

impl InMemoryPaymentRouter {
	pub fn new(
		default_key: PaymentProcessorKey,
		fallback_key: PaymentProcessorKey,
	) -> Self {
		Self {
			default_processor:  Arc::new(RwLock::new(PaymentProcessor {
				key:               Cow::Owned(default_key),
				health:            HealthStatus::Failing,
				min_response_time: 0,
			})),
			fallback_processor: Arc::new(RwLock::new(PaymentProcessor {
				key:               Cow::Owned(fallback_key),
				health:            HealthStatus::Failing,
				min_response_time: 0,
			})),
			default_breaker:    CircuitBreaker::<
				DefaultPolicy,
				PaymentProcessingError,
			>::builder()
			.build(),
			fallback_breaker:   CircuitBreaker::<
				DefaultPolicy,
				PaymentProcessingError,
			>::builder()
			.build(),
		}
	}

	pub fn update_processor_health(&self, processor: PaymentProcessor) {
		match processor.key.name {
			"default" => {
				*self.default_processor.write().unwrap() = processor;
			}
			"fallback" => {
				*self.fallback_processor.write().unwrap() = processor;
			}
			_ => {}
		}
	}
}

impl Default for InMemoryPaymentRouter {
	fn default() -> Self {
		Self::new(
			PaymentProcessorKey::new("default", "".into()),
			PaymentProcessorKey::new("fallback", "".into()),
		)
	}
}

#[async_trait]
impl PaymentRouter for InMemoryPaymentRouter {
	async fn get_processor_for_payment(
		&self,
	) -> Option<(
		Cow<'static, PaymentProcessorKey>,
		CircuitBreaker<DefaultPolicy, PaymentProcessingError>,
	)> {
		let default_processor = self.default_processor.read().unwrap();

		if default_processor.health.is_healthy() &&
			default_processor.min_response_time < 100 &&
			!matches!(
				self.default_breaker.current_state(),
				circuitbreaker_rs::State::Open
			) {
			return Some((
				default_processor.key.clone(),
				self.default_breaker.clone(),
			));
		}

		let fallback_processor = self.fallback_processor.read().unwrap();

		if fallback_processor.health.is_healthy() &&
			fallback_processor.min_response_time < 100 &&
			!matches!(
				self.fallback_breaker.current_state(),
				circuitbreaker_rs::State::Open
			) {
			return Some((
				fallback_processor.key.clone(),
				self.fallback_breaker.clone(),
			));
		}

		None
	}
}

#[cfg(test)]
mod tests {
	use std::borrow::Cow;

	use circuitbreaker_rs::State;
	use rinha_de_backend::domain::health_status::HealthStatus;
	use rinha_de_backend::domain::payment_processor::{
		PaymentProcessor, PaymentProcessorKey,
	};
	use rinha_de_backend::domain::payment_router::PaymentRouter;
	use rinha_de_backend::infrastructure::routing::in_memory_payment_router::InMemoryPaymentRouter;

	#[tokio::test]
	async fn test_get_processor_for_payment_default_healthy() {
		let router = InMemoryPaymentRouter::default();
		let default_processor = PaymentProcessor {
			key:               Cow::Owned(PaymentProcessorKey::new(
				"default",
				"http://default.com".into(),
			)),
			health:            HealthStatus::Healthy,
			min_response_time: 50,
		};
		router.update_processor_health(default_processor.clone());

		let (key, breaker) = router.get_processor_for_payment().await.unwrap();
		assert_eq!(key.url, default_processor.key.url);
		assert_eq!(key.name, default_processor.key.name);
		assert_eq!(breaker.current_state(), State::Closed);
	}

	#[tokio::test]
	async fn test_get_processor_for_payment_default_unhealthy() {
		let router = InMemoryPaymentRouter::default();
		let default_processor = PaymentProcessor {
			key:               Cow::Owned(PaymentProcessorKey::new(
				"default",
				"http://default.com".into(),
			)),
			health:            HealthStatus::Failing,
			min_response_time: 50,
		};
		router.update_processor_health(default_processor.clone());

		let result = router.get_processor_for_payment().await;
		assert!(result.is_none());
	}

	#[tokio::test]
	async fn test_get_processor_for_payment_default_slow() {
		let router = InMemoryPaymentRouter::default();
		let default_processor = PaymentProcessor {
			key:               Cow::Owned(PaymentProcessorKey::new(
				"default",
				"http://default.com".into(),
			)),
			health:            HealthStatus::Healthy,
			min_response_time: 150, // Too slow
		};
		router.update_processor_health(default_processor.clone());

		let result = router.get_processor_for_payment().await;
		assert!(result.is_none());
	}

	#[tokio::test]
	async fn test_get_processor_for_payment_default_circuit_open() {
		let router = InMemoryPaymentRouter::default();
		let default_processor = PaymentProcessor {
			key:               Cow::Owned(PaymentProcessorKey::new(
				"default",
				"http://default.com".into(),
			)),
			health:            HealthStatus::Healthy,
			min_response_time: 50,
		};
		router.update_processor_health(default_processor.clone());

		router.default_breaker.force_open();

		let result = router.get_processor_for_payment().await;
		assert!(result.is_none());
	}

	#[tokio::test]
	async fn test_get_processor_for_payment_fallback_healthy() {
		let router = InMemoryPaymentRouter::default();
		let fallback_processor = PaymentProcessor {
			key:               Cow::Owned(PaymentProcessorKey::new(
				"fallback",
				"http://fallback.com".into(),
			)),
			health:            HealthStatus::Healthy,
			min_response_time: 50,
		};
		router.update_processor_health(fallback_processor.clone());

		// Ensure default is not chosen
		let default_processor = PaymentProcessor {
			key:               Cow::Owned(PaymentProcessorKey::new(
				"default",
				"http://default.com".into(),
			)),
			health:            HealthStatus::Failing, // Make default unhealthy
			min_response_time: 50,
		};
		router.update_processor_health(default_processor.clone());

		let (key, breaker) = router.get_processor_for_payment().await.unwrap();
		assert_eq!(key.url, fallback_processor.key.url);
		assert_eq!(key.name, fallback_processor.key.name);
		assert_eq!(breaker.current_state(), State::Closed);
	}

	#[tokio::test]
	async fn test_get_processor_for_payment_no_processors() {
		let router = InMemoryPaymentRouter::default();
		let result = router.get_processor_for_payment().await;
		assert!(result.is_none());
	}

	#[tokio::test]
	async fn test_update_processor_health() {
		let router = InMemoryPaymentRouter::default();
		let processor = PaymentProcessor {
			key:               Cow::Owned(PaymentProcessorKey::new(
				"default",
				"http://test.com".into(),
			)),
			health:            HealthStatus::Healthy,
			min_response_time: 100,
		};
		router.update_processor_health(processor.clone());

		let default_processor = router.default_processor.read().unwrap();
		assert_eq!(default_processor.key.name, "default");
		assert_eq!(default_processor.key.url, "http://test.com");
	}
}

use std::sync::{Arc, RwLock};
use std::time::Duration;

use async_trait::async_trait;
use circuitbreaker_rs::{CircuitBreaker, DefaultPolicy, State};

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
		default_key: Arc<PaymentProcessorKey>,
		fallback_key: Arc<PaymentProcessorKey>,
	) -> Self {
		Self {
			default_processor:  Arc::new(RwLock::new(PaymentProcessor {
				key:               default_key,
				health:            HealthStatus::Failing,
				min_response_time: 0,
			})),
			fallback_processor: Arc::new(RwLock::new(PaymentProcessor {
				key:               fallback_key,
				health:            HealthStatus::Failing,
				min_response_time: 0,
			})),
			default_breaker:    CircuitBreaker::<
				DefaultPolicy,
				PaymentProcessingError,
			>::builder()
			.failure_threshold(0.5)
			.min_throughput(5)
			.probe_interval(10)
			.cooldown(Duration::from_secs(3))
			.build(),
			fallback_breaker:   CircuitBreaker::<
				DefaultPolicy,
				PaymentProcessingError,
			>::builder()
			.failure_threshold(0.1)
			.cooldown(Duration::from_secs(10))
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
			Arc::new(PaymentProcessorKey::new("default", "".into())),
			Arc::new(PaymentProcessorKey::new("fallback", "".into())),
		)
	}
}

#[async_trait]
impl PaymentRouter for InMemoryPaymentRouter {
	async fn get_processor_for_payment(
		&self,
	) -> Option<(
		Arc<PaymentProcessorKey>,
		CircuitBreaker<DefaultPolicy, PaymentProcessingError>,
	)> {
		let default_processor = self.default_processor.read().unwrap();
		let default_breaker_is_open =
			matches!(self.default_breaker.current_state(), State::Open);

		if default_processor.health.is_healthy() &&
			default_processor.min_response_time < 100 &&
			!default_breaker_is_open
		{
			return Some((
				default_processor.key.clone(),
				self.default_breaker.clone(),
			));
		}

		// Only consider fallback if the default's circuit breaker is open
		if default_breaker_is_open {
			let fallback_processor = self.fallback_processor.read().unwrap();
			if fallback_processor.health.is_healthy() &&
				fallback_processor.min_response_time < 100 &&
				!matches!(self.fallback_breaker.current_state(), State::Open)
			{
				return Some((
					fallback_processor.key.clone(),
					self.fallback_breaker.clone(),
				));
			}
		}

		// If default is just slow/unhealthy but the breaker is not open,
		// return None to force a re-queue and wait for it to recover.
		None
	}
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;

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
			key:               Arc::new(PaymentProcessorKey::new(
				"default",
				"http://default.com".into(),
			)),
			health:            HealthStatus::Healthy,
			min_response_time: 50,
		};
		router.update_processor_health(default_processor.clone());

		let (key, breaker) = router.get_processor_for_payment().await.unwrap();
		assert_eq!(key.name, "default");
		assert_eq!(breaker.current_state(), State::Closed);
	}

	#[tokio::test]
	async fn test_get_processor_for_payment_default_unhealthy_but_breaker_closed_waits()
	 {
		let router = InMemoryPaymentRouter::default();
		// Default is unhealthy
		router.update_processor_health(PaymentProcessor {
			key:               Arc::new(PaymentProcessorKey::new(
				"default",
				"http://default.com".into(),
			)),
			health:            HealthStatus::Failing,
			min_response_time: 50,
		});
		// Fallback is healthy
		router.update_processor_health(PaymentProcessor {
			key:               Arc::new(PaymentProcessorKey::new(
				"fallback",
				"http://fallback.com".into(),
			)),
			health:            HealthStatus::Healthy,
			min_response_time: 50,
		});

		// Should return None to wait for default to recover, since its breaker is
		// not open
		let result = router.get_processor_for_payment().await;
		assert!(result.is_none());
	}

	#[tokio::test]
	async fn test_get_processor_for_payment_default_slow_but_breaker_closed_waits() {
		let router = InMemoryPaymentRouter::default();
		// Default is slow
		router.update_processor_health(PaymentProcessor {
			key:               Arc::new(PaymentProcessorKey::new(
				"default",
				"http://default.com".into(),
			)),
			health:            HealthStatus::Healthy,
			min_response_time: 150, // Too slow
		});
		// Fallback is healthy
		router.update_processor_health(PaymentProcessor {
			key:               Arc::new(PaymentProcessorKey::new(
				"fallback",
				"http://fallback.com".into(),
			)),
			health:            HealthStatus::Healthy,
			min_response_time: 50,
		});

		// Should return None to wait for default to recover
		let result = router.get_processor_for_payment().await;
		assert!(result.is_none());
	}

	#[tokio::test]
	async fn test_get_processor_for_payment_default_circuit_open_uses_fallback() {
		let router = InMemoryPaymentRouter::default();
		// Default is healthy, but its breaker is open
		router.update_processor_health(PaymentProcessor {
			key:               Arc::new(PaymentProcessorKey::new(
				"default",
				"http://default.com".into(),
			)),
			health:            HealthStatus::Healthy,
			min_response_time: 50,
		});
		router.default_breaker.force_open();

		// Fallback is healthy
		let fallback_processor = PaymentProcessor {
			key:               Arc::new(PaymentProcessorKey::new(
				"fallback",
				"http://fallback.com".into(),
			)),
			health:            HealthStatus::Healthy,
			min_response_time: 50,
		};
		router.update_processor_health(fallback_processor.clone());

		// Should return fallback
		let (key, breaker) = router.get_processor_for_payment().await.unwrap();
		assert_eq!(key.name, "fallback");
		assert_eq!(breaker.current_state(), State::Closed);
	}

	#[tokio::test]
	async fn test_get_processor_for_payment_default_circuit_open_fallback_unhealthy()
	{
		let router = InMemoryPaymentRouter::default();
		// Default is healthy, but its breaker is open
		router.update_processor_health(PaymentProcessor {
			key:               Arc::new(PaymentProcessorKey::new(
				"default",
				"http://default.com".into(),
			)),
			health:            HealthStatus::Healthy,
			min_response_time: 50,
		});
		router.default_breaker.force_open();

		// Fallback is unhealthy
		router.update_processor_health(PaymentProcessor {
			key:               Arc::new(PaymentProcessorKey::new(
				"fallback",
				"http://fallback.com".into(),
			)),
			health:            HealthStatus::Failing,
			min_response_time: 50,
		});

		// Should return None
		let result = router.get_processor_for_payment().await;
		assert!(result.is_none());
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
			key:               Arc::new(PaymentProcessorKey::new(
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

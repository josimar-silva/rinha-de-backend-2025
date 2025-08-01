use std::borrow::Cow;

use reqwest::Client;
use rinha_de_backend::domain::health_status::HealthStatus;
use rinha_de_backend::domain::payment_processor::{
	PaymentProcessor, PaymentProcessorKey,
};
use rinha_de_backend::infrastructure::routing::in_memory_payment_router::InMemoryPaymentRouter;
use rinha_de_backend::infrastructure::workers::processor_health_monitor_worker::processor_health_monitor_worker;
use tokio::time::{Duration, sleep};

mod support;

use crate::support::payment_processor_container::setup_payment_processors;

#[tokio::test]
async fn test_update_processor_health_when_processor_is_reachable() {
	let (default_processor_container, fallback_processor_container) =
		setup_payment_processors().await;
	let default_url = default_processor_container.url.clone();
	let fallback_url = fallback_processor_container.url.clone();
	let default_key =
		PaymentProcessorKey::new("default", default_url.clone().into());
	let fallback_key =
		PaymentProcessorKey::new("fallback", fallback_url.clone().into());

	let http_client = Client::builder()
		.timeout(Duration::from_secs(2))
		.build()
		.unwrap();
	let router = InMemoryPaymentRouter::new(default_key, fallback_key);

	let processor_keys = vec![
		PaymentProcessorKey::new("default", default_url.into()),
		PaymentProcessorKey::new("fallback", fallback_url.into()),
	];

	// Spawn the worker
	let worker_handle = tokio::spawn(processor_health_monitor_worker(
		router.clone(),
		http_client.clone(),
		processor_keys,
	));

	wait_for_workflow_to_run().await;

	let default_processor = router
		.default_processor
		.read()
		.expect("Default processor not found");

	assert_eq!(default_processor.health, HealthStatus::Healthy);

	let fallback_processor = router
		.fallback_processor
		.read()
		.expect("Fallback processor not found");

	assert_eq!(fallback_processor.health, HealthStatus::Healthy);

	worker_handle.abort();
}

#[tokio::test]
async fn test_marks_processor_as_failing_when_unreachable() {
	let http_client = Client::builder()
		.timeout(Duration::from_secs(2))
		.build()
		.unwrap();
	let default_url = "http://non-existent-default:8080".to_string();
	let fallback_url = "http://non-existent-fallback:8080".to_string();
	let default_key =
		PaymentProcessorKey::new("default", default_url.clone().into());
	let fallback_key =
		PaymentProcessorKey::new("fallback", fallback_url.clone().into());

	let router =
		InMemoryPaymentRouter::new(default_key.clone(), fallback_key.clone());

	router.update_processor_health(PaymentProcessor {
		key:               Cow::Owned(default_key),
		health:            HealthStatus::Healthy,
		min_response_time: 0,
	});
	router.update_processor_health(PaymentProcessor {
		key:               Cow::Owned(fallback_key),
		health:            HealthStatus::Healthy,
		min_response_time: 0,
	});

	let processor_keys = vec![
		PaymentProcessorKey::new("default", default_url.into()),
		PaymentProcessorKey::new("fallback", fallback_url.into()),
	];

	let worker_handle = tokio::spawn(processor_health_monitor_worker(
		router.clone(),
		http_client.clone(),
		processor_keys,
	));

	wait_for_workflow_to_run().await;

	let default_processor = router
		.default_processor
		.read()
		.expect("Default processor not found");

	assert_eq!(default_processor.health, HealthStatus::Failing);

	let fallback_processor = router
		.fallback_processor
		.read()
		.expect("Fallback processor not found");

	assert_eq!(fallback_processor.health, HealthStatus::Failing);

	worker_handle.abort();
}

#[tokio::test]
async fn test_should_not_panic_an_error_occurs() {
	let http_client = Client::builder()
		.timeout(Duration::from_secs(2))
		.build()
		.unwrap();

	let default_key = PaymentProcessorKey::new(
		"default",
		"http://another-non-existent-default:8080".into(),
	);
	let fallback_key = PaymentProcessorKey::new(
		"fallback",
		"http://another-non-existent-fallback:8080".into(),
	);
	let router =
		InMemoryPaymentRouter::new(default_key.clone(), fallback_key.clone());

	router.update_processor_health(PaymentProcessor {
		key:               Cow::Owned(default_key),
		health:            HealthStatus::Healthy,
		min_response_time: 0,
	});
	router.update_processor_health(PaymentProcessor {
		key:               Cow::Owned(fallback_key),
		health:            HealthStatus::Healthy,
		min_response_time: 0,
	});

	let default_non_existent_url =
		"http://another-non-existent-default:8080".to_string();
	let fallback_non_existent_url =
		"http://another-non-existent-fallback:8080".to_string();

	let processor_keys = vec![
		PaymentProcessorKey::new("default", default_non_existent_url.into()),
		PaymentProcessorKey::new("fallback", fallback_non_existent_url.into()),
	];

	let worker_handle = tokio::spawn(processor_health_monitor_worker(
		router.clone(),
		http_client.clone(),
		processor_keys,
	));

	wait_for_workflow_to_run().await;

	assert!(!worker_handle.is_finished());

	worker_handle.abort();
}

async fn wait_for_workflow_to_run() {
	sleep(Duration::from_secs(6)).await;
}

use redis::AsyncCommands;
use reqwest::Client;
use rinha_de_backend::infrastructure::config::redis::{
	DEFAULT_PROCESSOR_HEALTH_KEY, FALLBACK_PROCESSOR_HEALTH_KEY,
};
use rinha_de_backend::infrastructure::persistence::redis_payment_processor_repository::RedisPaymentProcessorRepository;
use rinha_de_backend::infrastructure::workers::health_check_worker::health_check_worker;
use rinha_de_backend::use_cases::health_check::HealthCheckUseCase;
use tokio::time::Duration;

mod support;

use crate::support::payment_processor_container::setup_payment_processors;
use crate::support::redis_container::get_test_redis_client;

#[tokio::test]
async fn test_health_check_worker_success() {
	let redis_container = get_test_redis_client().await;
	let redis_client = redis_container.client.clone();
	let (default_processor_container, fallback_processor_container) =
		setup_payment_processors().await;
	let default_url = default_processor_container.url.clone();
	let fallback_url = fallback_processor_container.url.clone();
	let http_client = Client::new();

	let processor_repo = RedisPaymentProcessorRepository::new(redis_client.clone());
	let health_check_use_case =
		HealthCheckUseCase::new(processor_repo.clone(), http_client.clone());

	let worker_handle = tokio::spawn(health_check_worker(
		health_check_use_case,
		default_url.clone(),
		fallback_url.clone(),
	));

	// Give the worker some time to run and update Redis
	tokio::time::sleep(Duration::from_secs(30)).await;

	let mut con = redis_client
		.get_multiplexed_async_connection()
		.await
		.unwrap();
	let default_failing: i32 = con.hget("health:default", "failing").await.unwrap();
	let _default_min_response_time: u64 = con
		.hget(DEFAULT_PROCESSOR_HEALTH_KEY, "min_response_time")
		.await
		.unwrap();

	assert_eq!(default_failing, 0);

	let _fallback_min_response_time: u64 = con
		.hget(FALLBACK_PROCESSOR_HEALTH_KEY, "min_response_time")
		.await
		.unwrap();

	// Abort the worker to clean up
	worker_handle.abort();
}

#[tokio::test]
async fn test_health_check_worker_redis_failure() {
	let redis_container = get_test_redis_client().await;
	let redis_client = redis_container.client.clone();
	let redis_node = redis_container.container;
	let http_client = Client::new();

	let processor_repo = RedisPaymentProcessorRepository::new(redis_client.clone());
	let health_check_use_case =
		HealthCheckUseCase::new(processor_repo.clone(), http_client.clone());

	// Stop the redis container to simulate a connection failure
	let _ = redis_node.stop().await;

	let worker_handle = tokio::spawn(health_check_worker(
		health_check_use_case,
		"http://localhost:8080".to_string(),
		"http://localhost:8080".to_string(),
	));

	// Give the worker some time to run
	tokio::time::sleep(Duration::from_secs(6)).await;

	// The worker should not panic and should still be running
	assert!(!worker_handle.is_finished());

	// Abort the worker to clean up
	worker_handle.abort();
}

#[tokio::test]
async fn test_health_check_worker_http_failure() {
	let redis_container = get_test_redis_client().await;
	let redis_client = redis_container.client.clone();
	let http_client = Client::new();

	let processor_repo = RedisPaymentProcessorRepository::new(redis_client.clone());
	let health_check_use_case =
		HealthCheckUseCase::new(processor_repo.clone(), http_client.clone());

	// Use a non-existent URL to simulate HTTP failure
	let worker_handle = tokio::spawn(health_check_worker(
		health_check_use_case,
		"http://non-existent-url:8080".to_string(),
		"http://non-existent-url:8080".to_string(),
	));

	// Give the worker some time to attempt the HTTP call and update Redis
	tokio::time::sleep(Duration::from_secs(10)).await;

	let mut con = redis_client
		.get_multiplexed_async_connection()
		.await
		.unwrap();
	let default_failing: i32 = con
		.hget(DEFAULT_PROCESSOR_HEALTH_KEY, "failing")
		.await
		.unwrap_or(0);
	let fallback_failing: i32 = con
		.hget(FALLBACK_PROCESSOR_HEALTH_KEY, "failing")
		.await
		.unwrap_or(0);

	assert_eq!(default_failing, 1);
	assert_eq!(fallback_failing, 1);

	// Abort the worker to clean up
	worker_handle.abort();
}

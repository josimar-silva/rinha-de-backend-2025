use std::sync::Arc;

use rinha_de_backend::infrastructure::config::settings::Config;
use tokio::sync::mpsc;

mod support;
use crate::support::redis_container::get_test_redis_client;

#[cfg(test)]
#[actix_web::test]
async fn test_run_bind_error() {
	let listener = std::net::TcpListener::bind("0.0.0.0:9999").unwrap();

	let redis_container = get_test_redis_client().await;
	let redis = redis_container.get_redis().await;

	let dummy_config = Arc::new(Config {
		redis_url: "redis://127.0.0.1/".into(),
		default_payment_processor_url: "http://localhost:8080".into(),
		fallback_payment_processor_url: "http://localhost:8081".into(),
		server_keepalive: 60,
		report_url: None,
		payment_processor_worker_count: 4,
	});

	// Create a dummy MPSC channel for the test
	let (sender, _receiver) = mpsc::channel(1);

	// Attempt to bind to the same address, which should fail
	assert!(
		rinha_de_backend::run(dummy_config, sender, Arc::new(redis))
			.await
			.is_err()
	);
	drop(listener);
}

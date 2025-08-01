use std::sync::Arc;

use log::error;
use reqwest::Client;
use tokio::time::{Duration, sleep};

use crate::domain::health_status::HealthStatus;
use crate::domain::payment_processor::{PaymentProcessor, PaymentProcessorKey};
use crate::infrastructure::routing::in_memory_payment_router::InMemoryPaymentRouter;

pub async fn processor_health_monitor_worker(
	router: InMemoryPaymentRouter,
	http_client: Client,
) {
	let processor_keys = [
		Arc::clone(&router.default_processor.read().unwrap().key),
		Arc::clone(&router.fallback_processor.read().unwrap().key),
	];

	loop {
		for key in &processor_keys {
			compute_processor_health(&router, &http_client, Arc::clone(key)).await;
		}

		// Respect the 5-second rate limit for health checks
		sleep(Duration::from_secs(5)).await;
	}
}

async fn compute_processor_health(
	router: &InMemoryPaymentRouter,
	http_client: &Client,
	key: Arc<PaymentProcessorKey>,
) {
	let health_url = format!("{}/payments/service-health", key.url);

	match http_client.get(&health_url).send().await {
		Ok(resp) => {
			if resp.status().is_success() {
				match resp.json::<serde_json::Value>().await {
					Ok(json) => {
						let failing = json["failing"].as_bool().unwrap_or(true);
						let min_response_time =
							json["minResponseTime"].as_i64().unwrap_or(0) as u64;

						let health_status = if failing {
							HealthStatus::Failing
						} else {
							HealthStatus::Healthy
						};

						router.update_processor_health(PaymentProcessor {
							key: Arc::clone(&key),
							health: health_status.clone(),
							min_response_time,
						});
					}
					Err(e) => {
						error!(
							"Failed to parse health check response for {}: {e}",
							key.name
						);
					}
				}
			} else {
				router.update_processor_health(PaymentProcessor {
					key:               Arc::clone(&key),
					health:            HealthStatus::Failing,
					min_response_time: 0,
				});
			}
		}
		Err(e) => {
			error!("Failed to perform health check for {}: {e}", key.name);
			let processor = PaymentProcessor {
				key:               Arc::clone(&key),
				health:            HealthStatus::Failing,
				min_response_time: 0,
			};
			router.update_processor_health(processor);
		}
	}
}

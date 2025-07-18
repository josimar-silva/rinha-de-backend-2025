use std::sync::Arc;
use std::time::Duration;

use actix_web::{App, HttpServer, web};
use log::info;
use reqwest::Client;

pub mod adapters;
pub mod domain;
pub mod infrastructure;
pub mod use_cases;

use crate::adapters::web::handlers::{payments, payments_summary};
use crate::infrastructure::config::settings::Config;
use crate::infrastructure::persistence::redis_payment_processor_repository::RedisPaymentProcessorRepository;
use crate::infrastructure::persistence::redis_payment_repository::RedisPaymentRepository;
use crate::infrastructure::queue::redis_payment_queue::PaymentQueue;
use crate::infrastructure::workers::health_check_worker::health_check_worker;
use crate::infrastructure::workers::payment_processor_worker::payment_processing_worker;
use crate::use_cases::create_payment::CreatePaymentUseCase;
use crate::use_cases::get_payment_summary::GetPaymentSummaryUseCase;
use crate::use_cases::health_check::HealthCheckUseCase;
use crate::use_cases::process_payment::ProcessPaymentUseCase;

pub async fn run(config: Arc<Config>) -> std::io::Result<()> {
	env_logger::init();

	let redis_client =
		redis::Client::open(config.redis_url.clone()).expect("Invalid Redis URL");

	let http_client = Client::new();

	info!("Starting health check worker...");
	let processor_repo = RedisPaymentProcessorRepository::new(redis_client.clone());
	let health_check_use_case =
		HealthCheckUseCase::new(processor_repo.clone(), http_client.clone());

	tokio::spawn(health_check_worker(
		health_check_use_case,
		config.default_payment_processor_url.clone(),
		config.fallback_payment_processor_url.clone(),
	));

	info!("Starting payment processing worker...");
	let payment_queue = PaymentQueue::new(redis_client.clone());
	let payment_repo = RedisPaymentRepository::new(redis_client.clone());
	let process_payment_use_case =
		ProcessPaymentUseCase::new(payment_repo.clone(), http_client.clone());

	tokio::spawn(payment_processing_worker(
		payment_queue.clone(),
		payment_repo.clone(),
		processor_repo.clone(),
		process_payment_use_case,
		config.default_payment_processor_url.clone(),
		config.fallback_payment_processor_url.clone(),
	));

	info!("Starting Actix-Web server on 0.0.0.0:9999...");

	let create_payment_use_case = CreatePaymentUseCase::new(payment_queue.clone());
	let get_payment_summary_use_case =
		GetPaymentSummaryUseCase::new(payment_repo.clone());

	HttpServer::new(move || {
		App::new()
			.app_data(web::Data::new(create_payment_use_case.clone()))
			.app_data(web::Data::new(get_payment_summary_use_case.clone()))
			.service(payments)
			.service(payments_summary)
	})
	.keep_alive(Duration::from_secs(config.server_keepalive))
	.bind(("0.0.0.0", 9999))?
	.run()
	.await
}

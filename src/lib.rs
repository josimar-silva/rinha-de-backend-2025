use std::sync::Arc;
use std::time::Duration;

use actix_web::{App, HttpServer, web};
use log::info;
use reqwest::Client;
use tokio::sync::mpsc;

pub mod adapters;
pub mod domain;
pub mod infrastructure;
pub mod use_cases;

use crate::adapters::web::handlers::{payments, payments_purge, payments_summary};
use crate::domain::payment::Payment;
use crate::domain::payment_producer::PaymentProducer;
use crate::infrastructure::config::settings::Config;
use crate::infrastructure::persistence::redis_payment_repository::RedisPaymentRepository;
use crate::infrastructure::queue::mpsc_payment_producer::MpscPaymentProducer;
use crate::infrastructure::queue::redis_payment_queue::PaymentQueue;
use crate::infrastructure::routing::in_memory_payment_router::InMemoryPaymentRouter;
use crate::infrastructure::workers::payment_processor_worker::payment_processing_worker;
use crate::infrastructure::workers::processor_health_monitor_worker::processor_health_monitor_worker;
use crate::use_cases::get_payment_summary::GetPaymentSummaryUseCase;
use crate::use_cases::process_payment::ProcessPaymentUseCase;
use crate::use_cases::purge_payments::PurgePaymentsUseCase;

pub async fn run(
	config: Arc<Config>,
	payment_sender: mpsc::Sender<Payment>,
) -> std::io::Result<()> {
	env_logger::init();

	let redis_client =
		redis::Client::open(config.redis_url.as_ref()).expect("Invalid Redis URL");

	let http_client = Client::new();

	info!("Starting health check worker...");

	let in_memory_router = InMemoryPaymentRouter::new(
		config.get_default_key(),
		config.get_fallback_key(),
	);

	tokio::spawn(processor_health_monitor_worker(
		in_memory_router.clone(),
		http_client.clone(),
	));

	info!("Starting payment processing worker...");
	let payment_queue = PaymentQueue::new(redis_client.clone());
	let payment_repo = RedisPaymentRepository::new(redis_client.clone());

	let process_payment_use_case =
		ProcessPaymentUseCase::new(payment_repo.clone(), http_client.clone());

	tokio::spawn(payment_processing_worker(
		payment_queue.clone(),
		payment_repo.clone(),
		process_payment_use_case,
		in_memory_router.clone(),
	));

	info!("Starting Actix-Web server on 0.0.0.0:9999...");

	let payment_producer = MpscPaymentProducer::new(payment_sender);
	let get_payment_summary_use_case =
		GetPaymentSummaryUseCase::new(payment_repo.clone());
	let purge_payments_use_case = PurgePaymentsUseCase::new(payment_repo.clone());

	HttpServer::new(move || {
		App::new()
			.app_data(web::Data::new(
				Box::new(payment_producer.clone()) as Box<dyn PaymentProducer>
			))
			.app_data(web::Data::new(get_payment_summary_use_case.clone()))
			.app_data(web::Data::new(purge_payments_use_case.clone()))
			.service(payments)
			.service(payments_summary)
			.service(payments_purge)
	})
	.keep_alive(Duration::from_secs(config.server_keepalive))
	.bind(("0.0.0.0", 9999))?
	.run()
	.await
}

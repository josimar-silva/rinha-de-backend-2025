use std::sync::Arc;

#[cfg(feature = "perf")]
use pprof::flamegraph::Options;
use rinha_de_backend::domain::payment::Payment;
use rinha_de_backend::infrastructure::config::settings::Config;
use rinha_de_backend::infrastructure::queue::redis_payment_queue::PaymentQueue;
use rinha_de_backend::infrastructure::workers::mpsc_to_redis_worker::mpsc_to_redis_worker;
use rinha_de_backend::run;
use rinha_de_backend::use_cases::create_payment::CreatePaymentUseCase;
use tokio::sync::mpsc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	#[cfg(feature = "perf")]
	let guard = pprof::ProfilerGuardBuilder::default()
		.frequency(1000)
		.blocklist(&["libc", "libgcc", "pthread", "vdso"])
		.build()
		.unwrap();

	let config = Arc::new(Config::load().expect("Failed to load configuration"));

	let redis_client =
		redis::Client::open(config.redis_url.clone().as_ref()).unwrap();
	let payment_queue = PaymentQueue::new(redis_client.clone());
	let create_payment_use_case = CreatePaymentUseCase::new(payment_queue.clone());

	let (payment_sender, payment_receiver) = mpsc::channel::<Payment>(100_000);

	tokio::spawn(mpsc_to_redis_worker(
		payment_receiver,
		create_payment_use_case.clone(),
	));

	let result = run(config.clone(), payment_sender).await;

	#[cfg(feature = "perf")]
	if let Ok(report) = guard.report().build() {
		if let Some(report_url) = &config.report_url {
			let path = format!("{report_url}/flamegraph.svg");
			let mut file = std::fs::File::create(&path)?;
			let mut options = Options::default();
			options.title = "rinha-de-backend".to_string();
			options.count_name = "samples".to_string();
			report
				.flamegraph_with_options(&mut file, &mut options)
				.unwrap();
		}
	}

	result
}

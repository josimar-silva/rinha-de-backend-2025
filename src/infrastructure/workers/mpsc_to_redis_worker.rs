use log::{error, info};
use tokio::sync::mpsc;

use crate::domain::payment::Payment;
use crate::domain::queue::Queue;
use crate::use_cases::create_payment::CreatePaymentUseCase;

pub async fn mpsc_to_redis_worker<Q>(
	mut receiver: mpsc::Receiver<Payment>,
	create_payment_use_case: CreatePaymentUseCase<Q>,
) where
	Q: Queue<Payment> + Clone + Send + Sync + 'static,
{
	info!("Starting MPSC to Redis worker...");
	loop {
		while let Some(payment) = receiver.recv().await {
			if let Err(e) = create_payment_use_case.execute(payment).await {
				error!("Failed to push payment to Redis queue: {e:?}");
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use std::io::Write;
	use std::sync::{Arc, Mutex};
	use std::time::Duration;

	use async_trait::async_trait;
	use rinha_de_backend::domain::payment::Payment;
	use rinha_de_backend::domain::queue::{Message, Queue};
	use rinha_de_backend::infrastructure::workers::mpsc_to_redis_worker::mpsc_to_redis_worker;
	use rinha_de_backend::use_cases::create_payment::CreatePaymentUseCase;
	use tokio::sync::mpsc;
	use tokio::time::timeout;
	use uuid::Uuid;

	struct TestLogger {
		output: Arc<Mutex<Vec<u8>>>,
	}

	impl log::Log for TestLogger {
		fn enabled(&self, metadata: &log::Metadata) -> bool {
			metadata.level() <= log::Level::Error
		}

		fn log(&self, record: &log::Record) {
			if self.enabled(record.metadata()) {
				let mut output = self.output.lock().unwrap();
				writeln!(output, "{} - {}", record.level(), record.args()).unwrap();
			}
		}

		fn flush(&self) {}
	}

	#[derive(Clone)]
	struct MockPaymentQueue {
		payments: Arc<Mutex<Vec<Message<Payment>>>>,
	}

	impl MockPaymentQueue {
		fn new() -> Self {
			Self {
				payments: Arc::new(Mutex::new(Vec::new())),
			}
		}
	}

	#[async_trait::async_trait]
	impl Queue<Payment> for MockPaymentQueue {
		async fn pop(
			&self,
		) -> Result<Option<Message<Payment>>, Box<dyn std::error::Error + Send>> {
			Ok(None)
		}

		async fn push(
			&self,
			message: Message<Payment>,
		) -> Result<(), Box<dyn std::error::Error + Send>> {
			self.payments.lock().unwrap().push(message);
			Ok(())
		}
	}

	#[derive(Clone)]
	struct MockFailingPaymentQueue;

	#[async_trait]
	impl Queue<Payment> for MockFailingPaymentQueue {
		async fn push(
			&self,
			_message: Message<Payment>,
		) -> Result<(), Box<dyn std::error::Error + Send>> {
			Err(Box::new(std::io::Error::other("Mock push error")))
		}

		async fn pop(
			&self,
		) -> Result<Option<Message<Payment>>, Box<dyn std::error::Error + Send>> {
			Ok(None)
		}
	}

	#[tokio::test]
	async fn test_mpsc_to_redis_worker_sends_payment_to_queue() {
		let (sender, receiver) = mpsc::channel(1);
		let mock_queue = MockPaymentQueue::new();
		let create_payment_use_case = CreatePaymentUseCase::new(mock_queue.clone());

		tokio::spawn(mpsc_to_redis_worker(receiver, create_payment_use_case));

		let payment = Payment {
			correlation_id: Uuid::new_v4(),
			amount:         100.00,
			requested_at:   None,
			processed_at:   None,
			processed_by:   None,
		};

		sender.send(payment.clone()).await.unwrap();

		timeout(Duration::from_secs(1), async {
			loop {
				if !mock_queue.payments.lock().unwrap().is_empty() {
					break;
				}
				tokio::time::sleep(Duration::from_millis(10)).await;
			}
		})
		.await
		.expect("Timeout waiting for payment to be pushed to the queue");

		let payments = mock_queue.payments.lock().unwrap();
		assert_eq!(payments.len(), 1);
		assert_eq!(payments[0].id, payment.correlation_id);
		assert_eq!(payments[0].body.correlation_id, payment.correlation_id);
		assert_eq!(payments[0].body.amount, payment.amount);
	}

	#[tokio::test]
	async fn test_mpsc_to_redis_worker_logs_error_on_push_failure() {
		let (sender, receiver) = mpsc::channel(1);
		let mock_failing_queue = MockFailingPaymentQueue;
		let create_payment_use_case =
			CreatePaymentUseCase::new(mock_failing_queue.clone());

		let _worker_handle =
			tokio::spawn(mpsc_to_redis_worker(receiver, create_payment_use_case));

		let payment = Payment {
			correlation_id: Uuid::new_v4(),
			amount:         100.0,
			requested_at:   None,
			processed_at:   None,
			processed_by:   None,
		};

		// Use a test logger to capture log output
		let log_output = Arc::new(Mutex::new(Vec::<u8>::new()));
		let logger = TestLogger {
			output: log_output.clone(),
		};
		log::set_boxed_logger(Box::new(logger)).unwrap();
		log::set_max_level(log::LevelFilter::Error);

		sender.send(payment.clone()).await.unwrap();

		// Give the worker some time to process and log the error
		tokio::time::sleep(Duration::from_millis(100)).await;

		let logs = String::from_utf8(log_output.lock().unwrap().clone()).unwrap();
		assert!(logs.contains(
			"ERROR - Failed to push payment to Redis queue: Custom { kind: Other, \
			 error: \"Mock push error\" }"
		));
	}
}

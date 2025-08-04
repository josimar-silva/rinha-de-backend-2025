use std::time::Duration;

use rinha_de_backend::domain::payment::Payment;
use rinha_de_backend::domain::queue::Queue;
use rinha_de_backend::infrastructure::queue::redis_payment_queue::PaymentQueue;
use rinha_de_backend::infrastructure::workers::mpsc_to_redis_worker::mpsc_to_redis_worker;
use rinha_de_backend::use_cases::create_payment::CreatePaymentUseCase;
use tokio::sync::mpsc;
use uuid::Uuid;

mod support;

use crate::support::redis_container::get_test_redis_client;

#[tokio::test]
async fn test_mpsc_to_redis_worker_happy_path() {
	// Arrange
	let redis_container = get_test_redis_client().await;
	let redis_client = redis_container.client.clone();
	let payment_queue = PaymentQueue::new(redis_client);
	let create_payment_use_case = CreatePaymentUseCase::new(payment_queue.clone());
	let (sender, receiver) = mpsc::channel(1);

	let worker_handle =
		tokio::spawn(mpsc_to_redis_worker(receiver, create_payment_use_case));

	let payment_to_process = Payment {
		correlation_id: Uuid::new_v4(),
		amount:         150.75,
		requested_at:   None,
		processed_at:   None,
		processed_by:   None,
	};

	// Act
	sender.send(payment_to_process.clone()).await.unwrap();

	// Allow time for the worker to process the message
	tokio::time::sleep(Duration::from_secs(1)).await;

	// Assert
	let message = payment_queue.pop().await.unwrap().unwrap();
	let received_payment: Payment = message.body;

	assert_eq!(
		received_payment.correlation_id,
		payment_to_process.correlation_id
	);
	assert_eq!(received_payment.amount, payment_to_process.amount);

	// Cleanup
	worker_handle.abort();
}

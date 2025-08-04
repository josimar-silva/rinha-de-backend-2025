use std::time::Duration;

use actix_web::{App, test, web};
use rinha_de_backend::adapters::web::handlers::payments;
use rinha_de_backend::adapters::web::schema::PaymentRequest;
use rinha_de_backend::domain::payment::Payment;
use rinha_de_backend::domain::payment_producer::PaymentProducer;
use rinha_de_backend::domain::queue::Queue;
use rinha_de_backend::infrastructure::queue::mpsc_payment_producer::MpscPaymentProducer;
use rinha_de_backend::infrastructure::queue::redis_payment_queue::PaymentQueue;
use rinha_de_backend::use_cases::create_payment::CreatePaymentUseCase;
use tokio::sync::mpsc;
use uuid::Uuid;

mod support;

use crate::support::redis_container::get_test_redis_client;

#[actix_web::test]
async fn test_payments_post_returns_success() {
	let redis_container = get_test_redis_client().await;
	let redis_client = redis_container.client.clone();
	let payment_queue = PaymentQueue::new(redis_client.clone());
	let create_payment_use_case = CreatePaymentUseCase::new(payment_queue.clone());

	let (payment_sender, mut payment_receiver) = mpsc::channel(1);
	let mpsc_payment_producer = MpscPaymentProducer::new(payment_sender);

	let app = test::init_service(
		App::new()
			.app_data(web::Data::new(
				Box::new(mpsc_payment_producer.clone()) as Box<dyn PaymentProducer>
			))
			.service(payments),
	)
	.await;

	let payment_req = PaymentRequest {
		correlation_id: Uuid::new_v4(),
		amount:         100.51,
	};

	let req = test::TestRequest::post()
		.uri("/payments")
		.set_json(&payment_req)
		.to_request();
	let resp = test::call_service(&app, req).await;

	assert!(resp.status().is_success());

	// Assert that the payment was sent to the MPSC channel
	let received_payment =
		tokio::time::timeout(Duration::from_secs(1), payment_receiver.recv())
			.await
			.expect("Did not receive payment from MPSC channel")
			.expect("Channel closed");

	assert_eq!(received_payment.correlation_id, payment_req.correlation_id);
	assert_eq!(received_payment.amount, payment_req.amount);

	// Now, simulate the worker pushing to Redis and verify
	create_payment_use_case
		.execute(received_payment)
		.await
		.unwrap();

	let message = payment_queue.pop().await.unwrap().unwrap();
	let deserialized_payment: Payment = message.body;

	assert_eq!(
		deserialized_payment.correlation_id,
		payment_req.correlation_id
	);
	assert_eq!(deserialized_payment.amount, payment_req.amount);
}

#[actix_web::test]
async fn test_payments_post_channel_closed() {
	let (payment_sender, mut payment_receiver) = mpsc::channel(1);
	let mpsc_payment_producer = MpscPaymentProducer::new(payment_sender);

	// Close the receiver to simulate a channel closed error
	payment_receiver.close();

	let app = test::init_service(
		App::new()
			.app_data(web::Data::new(
				Box::new(mpsc_payment_producer.clone()) as Box<dyn PaymentProducer>
			))
			.service(payments),
	)
	.await;

	let payment_req = PaymentRequest {
		correlation_id: Uuid::new_v4(),
		amount:         100.0,
	};

	let req = test::TestRequest::post()
		.uri("/payments")
		.set_json(&payment_req)
		.to_request();
	let resp = test::call_service(&app, req).await;

	assert!(resp.status().is_server_error());
}

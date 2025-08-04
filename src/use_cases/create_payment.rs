use crate::domain::payment::Payment;
use crate::domain::queue::{Message, Queue};

#[derive(Clone)]
pub struct CreatePaymentUseCase<Q: Queue<Payment>> {
	payment_queue: Q,
}

impl<Q: Queue<Payment>> CreatePaymentUseCase<Q> {
	pub fn new(payment_queue: Q) -> Self {
		Self { payment_queue }
	}

	pub async fn execute(
		&self,
		payment: Payment,
	) -> Result<(), Box<dyn std::error::Error + Send>> {
		self.payment_queue
			.push(Message::with(payment.correlation_id, payment))
			.await
	}
}

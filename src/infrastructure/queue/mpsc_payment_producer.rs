use async_trait::async_trait;
use tokio::sync::mpsc;

use crate::domain::payment::Payment;
use crate::domain::payment_producer::PaymentProducer;

#[derive(Clone)]
pub struct MpscPaymentProducer {
	sender: mpsc::Sender<Payment>,
}

impl MpscPaymentProducer {
	pub fn new(sender: mpsc::Sender<Payment>) -> Self {
		Self { sender }
	}
}

#[async_trait]
impl PaymentProducer for MpscPaymentProducer {
	async fn send(
		&self,
		payment: Payment,
	) -> Result<(), Box<dyn std::error::Error + Send>> {
		self.sender
			.send(payment)
			.await
			.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;
		Ok(())
	}
}

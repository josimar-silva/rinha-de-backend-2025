use async_trait::async_trait;

use crate::domain::payment::Payment;

#[async_trait]
pub trait PaymentProducer: Send + Sync + 'static {
	async fn send(
		&self,
		payment: Payment,
	) -> Result<(), Box<dyn std::error::Error + Send>>;
}

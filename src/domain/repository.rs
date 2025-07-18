use async_trait::async_trait;

use crate::domain::payment::Payment;
use crate::domain::payment_processor::PaymentProcessor;

#[async_trait]
pub trait PaymentRepository: Send + Sync + 'static {
	async fn save(
		&self,
		payment: Payment,
	) -> Result<(), Box<dyn std::error::Error + Send>>;
	async fn get_summary_by_group(
		&self,
		group: &str,
		from_ts: i64,
		to_ts: i64,
	) -> Result<(usize, f64), Box<dyn std::error::Error + Send>>;
	async fn get_payment_summary(
		&self,
		group: &str,
		payment_id: &str,
	) -> Result<Payment, Box<dyn std::error::Error + Send>>;
	async fn is_already_processed(
		&self,
		payment_id: &str,
	) -> Result<bool, Box<dyn std::error::Error + Send>>;
}

#[async_trait]
pub trait PaymentProcessorRepository: Send + Sync + 'static {
	async fn save(
		&self,
		processor: PaymentProcessor,
	) -> Result<(), Box<dyn std::error::Error + Send>>;
	async fn get_health_of(
		&self,
		processor_name: &str,
	) -> Result<i32, Box<dyn std::error::Error + Send>>;
}

use std::sync::Arc;

use async_trait::async_trait;
use circuitbreaker_rs::{CircuitBreaker, DefaultPolicy};

use crate::domain::payment_processor::PaymentProcessorKey;
use crate::use_cases::process_payment::PaymentProcessingError;

#[async_trait]
pub trait PaymentRouter: Send + Sync + 'static {
	async fn get_processor_for_payment(
		&self,
		attempts_on_default_processor: u8,
	) -> Option<(
		Arc<PaymentProcessorKey>,
		CircuitBreaker<DefaultPolicy, PaymentProcessingError>,
	)>;
}

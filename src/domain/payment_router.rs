use std::borrow::Cow;

use async_trait::async_trait;
use circuitbreaker_rs::{CircuitBreaker, DefaultPolicy};

use crate::domain::payment_processor::PaymentProcessorKey;
use crate::use_cases::process_payment::PaymentProcessingError;

#[async_trait]
pub trait PaymentRouter: Send + Sync + 'static {
	async fn get_processor_for_payment(
		&self,
	) -> Option<(
		Cow<'static, PaymentProcessorKey>,
		CircuitBreaker<DefaultPolicy, PaymentProcessingError>,
	)>;
}

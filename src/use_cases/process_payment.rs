use std::error::Error;
use std::fmt;

use circuitbreaker_rs::{BreakerError, CircuitBreaker, DefaultPolicy};
use log::error;
use reqwest::Client;
use time::OffsetDateTime;

use crate::domain::payment::Payment;
use crate::domain::repository::PaymentRepository;

#[derive(Debug)]
pub struct PaymentProcessingError(pub String);

impl fmt::Display for PaymentProcessingError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "Service error: {}", self.0)
	}
}

impl Error for PaymentProcessingError {}

impl From<Box<dyn Error + Send + Sync + 'static>> for PaymentProcessingError {
	fn from(err: Box<dyn Error + Send + Sync + 'static>) -> Self {
		PaymentProcessingError(err.to_string())
	}
}

#[derive(Clone)]
pub struct ProcessPaymentUseCase<R: PaymentRepository> {
	payment_repo: R,
	http_client:  Client,
}

impl<R: PaymentRepository> ProcessPaymentUseCase<R> {
	pub fn new(payment_repo: R, http_client: Client) -> Self {
		Self {
			payment_repo,
			http_client,
		}
	}

	pub async fn execute(
		&self,
		mut payment: Payment,
		processor_url: String,
		processed_by: String,
		circuit_breaker: &mut CircuitBreaker<DefaultPolicy, PaymentProcessingError>,
	) -> Result<bool, Box<dyn Error + Send>> {
		payment.requested_at = Some(OffsetDateTime::now_utc());

		let result: Result<bool, BreakerError<PaymentProcessingError>> =
			circuit_breaker
				.call_async(|| async {
					let response = self
						.http_client
						.post(format!("{processor_url}/payments"))
						.json(&payment)
						.send()
						.await
						.map_err(|e| PaymentProcessingError(e.to_string()))?;

					if response.status().is_success() {
						Ok(true)
					} else {
						error!(
							"Processor returned non-success status for {}: {}",
							payment.correlation_id,
							response.status()
						);

						if response.status().is_client_error() {
							return Ok(false);
						}

						Err(PaymentProcessingError(
							"Service unavailable".to_string(),
						))
					}
				})
				.await;

		match result {
			Ok(result) => {
				if !result {
					Ok(false)
				} else {
					payment.processed_at = Some(OffsetDateTime::now_utc());
					payment.processed_by = Some(processed_by);
					self.payment_repo.save(payment).await?;
					Ok(true)
				}
			}
			Err(BreakerError::Open) => Err(Box::new(PaymentProcessingError(
				"Circuit breaker open".to_string(),
			)) as Box<dyn Error + Send>),
			Err(BreakerError::Operation(e)) => {
				error!("Circuit breaker prevented execution: {e}");
				Err(Box::new(e) as Box<dyn Error + Send>)
			}
			Err(e) => {
				error!("Operation failed: {e}");
				Err(Box::new(e) as Box<dyn Error + Send>)
			}
		}
	}
}

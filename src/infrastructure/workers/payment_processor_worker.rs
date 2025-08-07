use std::time::Duration;

use circuitbreaker_rs::State;
use log::{error, info, warn};
use tokio::time::sleep;

use crate::domain::payment::Payment;
use crate::domain::payment_router::PaymentRouter;
use crate::domain::queue::Queue;
use crate::domain::repository::PaymentRepository;
use crate::use_cases::process_payment::ProcessPaymentUseCase;

pub async fn payment_processing_worker<Q, PR, R>(
	queue: Q,
	payment_repo: PR,
	process_payment_use_case: ProcessPaymentUseCase<PR>,
	router: R,
) where
	Q: Queue<Payment> + Clone + Send + Sync + 'static,
	PR: PaymentRepository + Clone + Send + Sync + 'static,
	R: PaymentRouter + Clone + Send + Sync + 'static,
{
	loop {
		let message = match queue.pop().await {
			Ok(Some(val)) => val,
			Ok(None) => {
				info!("No payments in queue, waiting...");
				sleep(Duration::from_secs(1)).await;
				continue;
			}
			Err(e) => {
				error!("Failed to pop from payments queue: {e}");
				sleep(Duration::from_secs(1)).await;
				continue;
			}
		};

		let message_id = message.id;

		info!("Started processing message with id '{}'", &message_id);

		let payment: Payment = message.body.clone();

		if payment_repo
			.is_already_processed(&payment.correlation_id.to_string())
			.await
			.unwrap_or(false)
		{
			info!("Payment already processed. Skipping it.");
			continue;
		}

		let mut processed = false;
		let mut attempts = 0;

		while !processed {
			attempts += 1;

			if let Some((key, mut circuit_breaker)) =
				router.get_processor_for_payment(attempts).await
			{
				if circuit_breaker.current_state() == State::Open {
					break;
				}

				processed = process_payment_use_case
					.execute(
						payment.clone(),
						key.url.to_string(),
						key.name.to_string(),
						&mut circuit_breaker,
					)
					.await
					.unwrap_or(false);
			} else {
				break;
			}
		}

		if !processed {
			warn!(
				"Payment {} could not be processed after all attempts. Re-queueing.",
				payment.correlation_id
			);
			if let Err(e) = queue.push(message).await {
				error!("Failed to re-queue payment: {e}");
			}
			sleep(Duration::from_millis(250)).await;
		}

		info!("Message with id '{}' processed.", &message_id);
	}
}

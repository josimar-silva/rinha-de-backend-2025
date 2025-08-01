use log::{error, info};
use tokio::sync::mpsc;

use crate::domain::payment::Payment;
use crate::domain::queue::Queue;
use crate::use_cases::create_payment::CreatePaymentUseCase;

pub async fn mpsc_to_redis_worker<Q>(
	mut receiver: mpsc::Receiver<Payment>,
	create_payment_use_case: CreatePaymentUseCase<Q>,
) where
	Q: Queue<Payment> + Clone + Send + Sync + 'static,
{
	info!("Starting MPSC to Redis worker...");
	loop {
		while let Some(payment) = receiver.recv().await {
			if let Err(e) = create_payment_use_case.execute(payment).await {
				error!("Failed to push payment to Redis queue: {e:?}");
			}
		}
	}
}

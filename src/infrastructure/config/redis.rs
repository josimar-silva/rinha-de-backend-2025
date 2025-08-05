use std::sync::Arc;

use redis::RedisError;
use redis::aio::MultiplexedConnection;

pub const PAYMENTS_QUEUE_KEY: &str = "payments_queue";
pub const PROCESSED_PAYMENTS_SET_KEY: &str = "processed_payments";
pub const DEFAULT_PAYMENT_SUMMARY_KEY: &str = "payment_summary:default";
pub const FALLBACK_PAYMENT_SUMMARY_KEY: &str = "payment_summary:fallback";

#[derive(Clone)]
pub struct Redis {
	pub connection: Arc<MultiplexedConnection>,
}

impl Redis {
	pub async fn new(redis_url: &str) -> Result<Self, RedisError> {
		let client = redis::Client::open(redis_url)?;
		let connection = client.get_multiplexed_async_connection().await?;
		Ok(Self {
			connection: Arc::new(connection),
		})
	}
}

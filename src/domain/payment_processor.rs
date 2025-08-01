use std::borrow::Cow;

use crate::domain::health_status::HealthStatus;

#[derive(Clone)]
pub struct PaymentProcessorKey {
	pub name: &'static str,
	pub url:  Cow<'static, str>,
}

impl PaymentProcessorKey {
	pub fn new(name: &'static str, url: Cow<'static, str>) -> Self {
		Self { name, url }
	}
}

#[derive(Clone)]
pub struct PaymentProcessor {
	pub key:               Cow<'static, PaymentProcessorKey>,
	pub health:            HealthStatus,
	pub min_response_time: u64,
}

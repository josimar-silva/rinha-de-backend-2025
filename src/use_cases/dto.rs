use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CreatePaymentCommand {
	pub correlation_id: Uuid,
	pub amount:         f64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GetPaymentSummaryQuery {
	pub from: Option<OffsetDateTime>,
	pub to:   Option<OffsetDateTime>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct PaymentSummaryResult {
	#[serde(rename = "totalRequests")]
	pub total_requests: usize,
	#[serde(rename = "totalAmount")]
	pub total_amount:   f64,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct PaymentsSummaryResponse {
	pub default:  PaymentSummaryResult,
	pub fallback: PaymentSummaryResult,
}

#[cfg(test)]
mod tests {
	use serde_json::json;

	use super::*;

	#[test]
	fn test_serialize_payments_summary_response() {
		let summary = PaymentsSummaryResponse {
			default:  PaymentSummaryResult {
				total_requests: 43236,
				total_amount:   415542345.98,
			},
			fallback: PaymentSummaryResult {
				total_requests: 423545,
				total_amount:   329347.34,
			},
		};

		let serialized = serde_json::to_value(&summary).unwrap();
		let expected = json!({
			"default": {
				"totalRequests": 43236,
				"totalAmount": 415542345.98
			},
			"fallback": {
				"totalRequests": 423545,
				"totalAmount": 329347.34
			}
		});

		assert_eq!(serialized, expected);
	}

	#[test]
	fn test_deserialize_payments_summary_response() {
		let json = json!({
			"default": {
				"totalRequests": 43236,
				"totalAmount": 415542345.98
			},
			"fallback": {
				"totalRequests": 423545,
				"totalAmount": 329347.34
			}
		});

		let deserialized: PaymentsSummaryResponse =
			serde_json::from_value(json).unwrap();
		let expected = PaymentsSummaryResponse {
			default:  PaymentSummaryResult {
				total_requests: 43236,
				total_amount:   415542345.98,
			},
			fallback: PaymentSummaryResult {
				total_requests: 423545,
				total_amount:   329347.34,
			},
		};

		assert_eq!(deserialized, expected);
	}
}

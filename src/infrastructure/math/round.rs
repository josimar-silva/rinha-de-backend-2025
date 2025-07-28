pub trait Round {
	fn round_decimals_to(self, decimals: i32) -> Self;
}

impl Round for f64 {
	fn round_decimals_to(self, decimals: i32) -> f64 {
		let shift_factor = 10_f64.powi(decimals);

		(self * shift_factor).round() / shift_factor
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_round_positive() {
		let value = 1.23456;
		assert_eq!(value.round_decimals_to(2), 1.23);
	}

	#[test]
	fn test_round_positive_up() {
		let value = 1.23678;
		assert_eq!(value.round_decimals_to(2), 1.24);
	}

	#[test]
	fn test_round_negative() {
		let value = -1.23456;
		assert_eq!(value.round_decimals_to(2), -1.23);
	}

	#[test]
	fn test_round_negative_up() {
		let value = -1.23678;
		assert_eq!(value.round_decimals_to(2), -1.24);
	}

	#[test]
	fn test_round_to_zero_decimals() {
		let value = 1.23456;
		assert_eq!(value.round_decimals_to(0), 1.0);
	}

	#[test]
	fn test_round_already_rounded() {
		let value = 1.23;
		assert_eq!(value.round_decimals_to(2), 1.23);
	}

	#[test]
	fn test_round_more_decimals() {
		let value = 1.23456789;
		assert_eq!(value.round_decimals_to(5), 1.23457);
	}

	#[test]
	fn test_round_less_decimals() {
		let value = 1.2;
		assert_eq!(value.round_decimals_to(5), 1.2);
	}
}

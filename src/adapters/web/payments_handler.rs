use actix_web::{HttpResponse, Responder, ResponseError, post, web};
use log::warn;

use crate::adapters::web::errors::ApiError;
use crate::adapters::web::schema::{PaymentRequest, PaymentResponse};
use crate::domain::payment::Payment;
use crate::domain::payment_producer::PaymentProducer;

#[post("/payments")]
pub async fn payments(
	payload: web::Json<PaymentRequest>,
	payment_producer: web::Data<Box<dyn PaymentProducer>>,
) -> impl Responder {
	let payment = Payment {
		correlation_id: payload.correlation_id,
		amount:         payload.amount,
		requested_at:   None,
		processed_at:   None,
		processed_by:   None,
	};

	match payment_producer.send(payment).await {
		Ok(_) => HttpResponse::Ok().json(PaymentResponse {
			payment: payload.0,
			status:  "queued".to_string(),
		}),
		Err(e) => {
			warn!("Error processing payment: {e:?}");
			ApiError::InternalServerError.error_response()
		}
	}
}

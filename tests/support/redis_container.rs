use redis::AsyncCommands;
use rinha_de_backend::config::{
	DEFAULT_PROCESSOR_HEALTH_KEY, FALLBACK_PROCESSOR_HEALTH_KEY, PAYMENTS_QUEUE_KEY,
	PROCESSED_PAYMENTS_SET_KEY,
};
use testcontainers::GenericImage;
use testcontainers::core::{ContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;

pub async fn get_test_redis_client()
-> (redis::Client, testcontainers::ContainerAsync<GenericImage>) {
	let container = GenericImage::new("redis", "8.0.3-alpine")
		.with_exposed_port(ContainerPort::Tcp(6379))
		.with_wait_for(WaitFor::message_on_stdout("Ready to accept connections"))
		.start()
		.await
		.unwrap();
	let host_port = container.get_host_port_ipv4(6379).await;
	let redis_url = format!("redis://127.0.0.1:{}", host_port.unwrap());
	let client = redis::Client::open(redis_url).expect("Invalid Redis URL");
	let mut con = client
		.get_multiplexed_async_connection()
		.await
		.expect("Failed to connect to Redis");
	// Clear Redis for a clean test environment
	let _: () = con
		.del(PAYMENTS_QUEUE_KEY)
		.await
		.expect("Failed to clear payments_queue");
	let _: () = con
		.del("payments_summary_default")
		.await
		.expect("Failed to clear payments_summary_default");
	let _: () = con
		.del("payments_summary_fallback")
		.await
		.expect("Failed to clear payments_summary_fallback");
	let _: () = con
		.del(PROCESSED_PAYMENTS_SET_KEY)
		.await
		.expect("Failed to clear processed_correlation_ids");
	let _: () = con
		.del(DEFAULT_PROCESSOR_HEALTH_KEY)
		.await
		.expect("Failed to clear health:default");
	let _: () = con
		.del(FALLBACK_PROCESSOR_HEALTH_KEY)
		.await
		.expect("Failed to clear health:fallback");
	(client, container)
}

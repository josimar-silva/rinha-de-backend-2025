use log::{error, info};
use redis::AsyncCommands;
use rinha_de_backend::infrastructure::config::redis::Redis;
use testcontainers::GenericImage;
use testcontainers::core::{ContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;

pub struct RedisTestContainer {
	pub client:    redis::Client,
	pub container: testcontainers::ContainerAsync<GenericImage>,
}

impl RedisTestContainer {
	pub fn client(&self) -> &redis::Client {
		&self.client
	}

	pub async fn get_redis(&self) -> Redis {
		let redis_url =
			format!("redis://{}", &self.client.get_connection_info().addr);

		Redis::new(&redis_url).await.unwrap()
	}
}

pub async fn get_test_redis_client() -> RedisTestContainer {
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

	match AsyncCommands::flushdb::<String>(&mut con).await {
		Ok(_) => {
			info!("Redis database cleared successfully.");
		}
		Err(e) => {
			error!("Redis database cleared failed: {e}");
		}
	}

	RedisTestContainer { client, container }
}

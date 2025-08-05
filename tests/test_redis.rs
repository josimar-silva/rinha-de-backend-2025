use redis::AsyncCommands;
use rinha_de_backend::infrastructure::config::redis::Redis;

mod support;

#[tokio::test]
async fn test_redis_new_connection() {
	let redis_container = support::redis_container::get_test_redis_client().await;
	let redis = redis_container.get_redis().await;

	let mut con = redis.connection.as_ref().clone();

	let _: () = con.set("test_key", "test_value").await.unwrap();
	let value: String = con.get("test_key").await.unwrap();

	assert_eq!(value, "test_value");
}

#[tokio::test]
async fn test_redis_connection_error() {
	let invalid_redis_url = "redis://127.0.0.1:9999"; // Assuming nothing is running on 9999
	let result = Redis::new(invalid_redis_url).await;
	assert!(result.is_err());
}

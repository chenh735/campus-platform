use redis::aio::ConnectionManager;
use redis::Client;

pub async fn init_redis(redis_url: &str) -> ConnectionManager {
    let client = Client::open(redis_url).expect("Failed to create Redis client");
    ConnectionManager::new(client)
        .await
        .expect("Failed to connect to Redis")
}

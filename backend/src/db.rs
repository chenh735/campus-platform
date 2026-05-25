use sqlx::mysql::MySqlPoolOptions;
use sqlx::MySqlPool;

pub async fn init_pool(database_url: &str) -> MySqlPool {
    MySqlPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .expect("Failed to connect to MySQL")
}

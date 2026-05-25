use sqlx::MySqlPool;

use crate::config::Config;

pub mod admin;
pub mod auth;
pub mod courses;
pub mod materials;
pub mod projects;
pub mod reviews;
pub mod upload;
pub mod users;

#[derive(Clone)]
pub struct AppState {
    pub pool: MySqlPool,
    pub config: Config,
    pub redis: RedisConn,
}

pub type RedisConn = redis::aio::ConnectionManager;

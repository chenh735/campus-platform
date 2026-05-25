use dotenvy::dotenv;
use std::env;

#[derive(Clone)]
pub struct Config {
    pub app_host: String,
    pub app_port: u16,
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub upload_dir: String,
    pub max_upload_mb: u64,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub smtp_from: String,
    pub allowed_email_domains: Vec<String>,
    pub cors_allowed_origins: Vec<String>,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv().ok();
        Self {
            app_host: env::var("APP_HOST").unwrap_or_else(|_| "127.0.0.1".into()),
            app_port: env::var("APP_PORT")
                .unwrap_or_else(|_| "8080".into())
                .parse()
                .unwrap_or(8080),
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            redis_url: env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".into()),
            jwt_secret: env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            upload_dir: env::var("UPLOAD_DIR").unwrap_or_else(|_| "./uploads".into()),
            max_upload_mb: env::var("MAX_UPLOAD_MB")
                .unwrap_or_else(|_| "50".into())
                .parse()
                .unwrap_or(50),
            smtp_host: env::var("SMTP_HOST").unwrap_or_default(),
            smtp_port: env::var("SMTP_PORT")
                .unwrap_or_else(|_| "465".into())
                .parse()
                .unwrap_or(465),
            smtp_username: env::var("SMTP_USERNAME").unwrap_or_default(),
            smtp_password: env::var("SMTP_PASSWORD").unwrap_or_default(),
            smtp_from: env::var("SMTP_FROM").unwrap_or_default(),
            allowed_email_domains: env::var("ALLOWED_EMAIL_DOMAINS")
                .unwrap_or_else(|_| "mail2.sysu.edu.cn".into())
                .split(',')
                .map(|s| s.trim().to_lowercase())
                .collect(),
            cors_allowed_origins: env::var("CORS_ALLOWED_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:3000,http://127.0.0.1:3000".into())
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
        }
    }
}

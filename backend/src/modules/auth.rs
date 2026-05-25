use axum::{extract::State, Json};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::middleware::auth::{AuthUser, Claims};
use crate::modules::AppState;

#[derive(Debug, Deserialize)]
pub struct SendCodeRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub code: String,
    pub nickname: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserInfo,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UserInfo {
    pub id: i64,
    pub email: String,
    pub nickname: String,
    pub role: String,
    pub experience: i32,
    pub status: String,
}

pub async fn record_user_activity(pool: &sqlx::MySqlPool, user_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO user_daily_activity (user_id, activity_date, last_active_at) \
         VALUES (?, CURRENT_DATE(), NOW()) \
         ON DUPLICATE KEY UPDATE last_active_at = VALUES(last_active_at)",
    )
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn send_code(
    State(state): State<AppState>,
    Json(req): Json<SendCodeRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let email = req.email.trim().to_lowercase();

    if !state
        .config
        .allowed_email_domains
        .iter()
        .any(|domain| email.ends_with(&format!("@{}", domain)))
    {
        return Err(AppError::BadRequest(format!(
            "仅支持 {} 邮箱注册",
            state.config.allowed_email_domains.join("、")
        )));
    }
    if [
        &state.config.smtp_host,
        &state.config.smtp_username,
        &state.config.smtp_password,
        &state.config.smtp_from,
    ]
    .iter()
    .any(|value| value.trim().is_empty())
    {
        return Err(AppError::Internal("邮件服务未配置，请联系管理员".into()));
    }

    // Check cooldown
    let cooldown_key = format!("email_code_cooldown:{}", email);
    let mut redis = state.redis.clone();
    let exists: bool = redis::Cmd::exists(&cooldown_key)
        .query_async::<_, bool>(&mut redis)
        .await
        .unwrap_or(false);
    if exists {
        return Err(AppError::BadRequest(
            "验证码发送过于频繁，请60秒后再试".into(),
        ));
    }

    // Generate code
    let code: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(6)
        .map(char::from)
        .map(|c| c.to_ascii_uppercase())
        .collect();

    // Store in Redis with 10-minute TTL
    let code_key = format!("email_code:register:{}", email);
    let mut redis = state.redis.clone();
    redis::Cmd::set_ex(&code_key, &code, 600_u64)
        .query_async::<_, ()>(&mut redis)
        .await
        .map_err(|e| AppError::Internal(format!("Redis 错误: {}", e)))?;

    // Set cooldown
    redis::Cmd::set_ex(&cooldown_key, "1", 60_u64)
        .query_async::<_, ()>(&mut redis)
        .await
        .map_err(|e| AppError::Internal(format!("Redis 错误: {}", e)))?;

    // Do not report success or retain cooldown state when delivery fails.
    if let Err(err) = send_email(&state.config, &email, &code).await {
        tracing::error!("邮件发送失败: {:?}", err);
        let _: () = redis::Cmd::del(&code_key)
            .query_async::<_, ()>(&mut redis)
            .await
            .unwrap_or(());
        let _: () = redis::Cmd::del(&cooldown_key)
            .query_async::<_, ()>(&mut redis)
            .await
            .unwrap_or(());
        return Err(AppError::Internal("验证码发送失败，请稍后重试".into()));
    }

    Ok(Json(serde_json::json!({
        "message": "验证码已发送"
    })))
}

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let email = req.email.trim().to_lowercase();

    if !state
        .config
        .allowed_email_domains
        .iter()
        .any(|domain| email.ends_with(&format!("@{}", domain)))
    {
        return Err(AppError::BadRequest(format!(
            "仅支持 {} 邮箱注册",
            state.config.allowed_email_domains.join("、")
        )));
    }

    if req.nickname.trim().is_empty() || req.nickname.len() > 64 {
        return Err(AppError::BadRequest("昵称长度应在1-64个字符".into()));
    }

    if req.password.len() < 6 {
        return Err(AppError::BadRequest("密码长度至少6位".into()));
    }

    // Verify code
    let code_key = format!("email_code:register:{}", email);
    let stored_code: Option<String> = {
        let mut redis = state.redis.clone();
        redis::Cmd::get(&code_key)
            .query_async::<_, Option<String>>(&mut redis)
            .await
            .unwrap_or(None)
    };

    match stored_code {
        Some(code) if code.to_uppercase() == req.code.to_uppercase() => {}
        Some(_) => return Err(AppError::BadRequest("验证码错误".into())),
        None => return Err(AppError::BadRequest("验证码已过期，请重新发送".into())),
    }

    // Check if email already exists
    let existing = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE email = ?")
        .bind(&email)
        .fetch_one(&state.pool)
        .await?;

    if existing > 0 {
        return Err(AppError::Conflict("该邮箱已注册".into()));
    }

    let password_hash =
        hash(&req.password, DEFAULT_COST).map_err(|_| AppError::Internal("密码加密失败".into()))?;

    let now = Utc::now().naive_utc();

    let result = sqlx::query(
        "INSERT INTO users (email, password_hash, nickname, role, experience, status, created_at, updated_at) VALUES (?, ?, ?, 'user', 0, 'active', ?, ?)"
    )
    .bind(&email)
    .bind(&password_hash)
    .bind(&req.nickname.trim())
    .bind(&now)
    .bind(&now)
    .execute(&state.pool)
    .await?;

    let user_id = result.last_insert_id() as i64;

    // Delete used code
    let mut redis = state.redis.clone();
    let _: () = redis::Cmd::del(&code_key)
        .query_async::<_, ()>(&mut redis)
        .await
        .unwrap_or(());

    let user = UserInfo {
        id: user_id,
        email,
        nickname: req.nickname.trim().to_string(),
        role: "user".into(),
        experience: 0,
        status: "active".into(),
    };

    let token = generate_token(user_id, "user", &state.config)?;
    record_user_activity(&state.pool, user_id).await?;

    Ok(Json(AuthResponse { token, user }))
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let email = req.email.trim().to_lowercase();

    let user = sqlx::query_as::<_, UserInfo>(
        "SELECT id, email, nickname, role, experience, status FROM users WHERE email = ?",
    )
    .bind(&email)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::Unauthorized("邮箱或密码错误".into()))?;

    if user.status == "disabled" {
        return Err(AppError::Forbidden("账号已被禁用".into()));
    }

    let password_hash: String = sqlx::query_scalar("SELECT password_hash FROM users WHERE id = ?")
        .bind(user.id)
        .fetch_one(&state.pool)
        .await?;

    let valid = verify(&req.password, &password_hash)
        .map_err(|_| AppError::Internal("密码校验失败".into()))?;

    if !valid {
        return Err(AppError::Unauthorized("邮箱或密码错误".into()));
    }

    let token = generate_token(user.id, &user.role, &state.config)?;
    record_user_activity(&state.pool, user.id).await?;

    Ok(Json(AuthResponse { token, user }))
}

pub async fn get_me(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<UserInfo>, AppError> {
    let user = sqlx::query_as::<_, UserInfo>(
        "SELECT id, email, nickname, role, experience, status FROM users WHERE id = ?",
    )
    .bind(auth.id)
    .fetch_one(&state.pool)
    .await?;

    if user.status == "disabled" {
        return Err(AppError::Forbidden("账号已被禁用".into()));
    }

    Ok(Json(user))
}

fn generate_token(
    user_id: i64,
    role: &str,
    config: &crate::config::Config,
) -> Result<String, AppError> {
    let now = Utc::now();
    let exp = (now + chrono::Duration::days(7)).timestamp() as usize;

    let claims = Claims {
        sub: user_id,
        role: role.to_string(),
        exp,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
    .map_err(|_| AppError::Internal("令牌生成失败".into()))
}

async fn send_email(config: &crate::config::Config, to: &str, code: &str) -> Result<(), AppError> {
    use lettre::{
        message::Message,
        transport::smtp::client::{Tls, TlsParameters},
        transport::smtp::{authentication::Credentials, AsyncSmtpTransport},
        AsyncTransport, Tokio1Executor,
    };

    let from: lettre::message::Mailbox = config
        .smtp_from
        .parse()
        .map_err(|e| AppError::Internal(format!("发件人地址无效: {}", e)))?;
    let to: lettre::message::Mailbox = to
        .parse()
        .map_err(|e| AppError::Internal(format!("收件人地址无效: {}", e)))?;

    let email = Message::builder()
        .from(from)
        .to(to.clone())
        .subject("校园协作平台 - 邮箱验证码")
        .body(format!(
            "您的验证码是：{}\n\n有效期 10 分钟，请勿泄露给他人。",
            code
        ))
        .map_err(|e| AppError::Internal(format!("构建邮件失败: {}", e)))?;

    let tls_params = TlsParameters::new(config.smtp_host.clone())
        .map_err(|e| AppError::Internal(format!("TLS 初始化失败: {}", e)))?;

    let transport =
        AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(config.smtp_host.as_str())
            .port(config.smtp_port)
            .tls(Tls::Wrapper(tls_params))
            .credentials(Credentials::new(
                config.smtp_username.clone(),
                config.smtp_password.clone(),
            ))
            .build();

    transport
        .send(email)
        .await
        .map_err(|e| AppError::Internal(format!("邮件发送失败: {}", e)))?;

    tracing::info!("验证码已发送至 {}", to);
    Ok(())
}

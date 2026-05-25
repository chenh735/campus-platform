use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    Json,
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;

use crate::modules::AppState;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: i64,
    pub role: String,
    pub exp: usize,
}

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: i64,
    pub role: String,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = (StatusCode, Json<serde_json::Value>);

    fn from_request_parts<'life0, 'life1, 'async_trait>(
        parts: &'life0 mut Parts,
        state: &'life1 AppState,
    ) -> Pin<Box<dyn Future<Output = Result<Self, Self::Rejection>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            let header = parts
                .headers
                .get("Authorization")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.strip_prefix("Bearer "))
                .ok_or_else(|| {
                    (
                        StatusCode::UNAUTHORIZED,
                        Json(serde_json::json!({ "error": "未提供认证令牌" })),
                    )
                })?;

            let token_data = decode::<Claims>(
                header,
                &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
                &Validation::new(Algorithm::HS256),
            )
            .map_err(|_| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({ "error": "令牌无效或已过期" })),
                )
            })?;

            let user = sqlx::query_as::<_, (String, String)>(
                "SELECT role, status FROM users WHERE id = ?",
            )
            .bind(token_data.claims.sub)
            .fetch_optional(&state.pool)
            .await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": "数据库错误" })),
                )
            })?
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({ "error": "用户不存在" })),
                )
            })?;

            if user.1 != "active" {
                return Err((
                    StatusCode::FORBIDDEN,
                    Json(serde_json::json!({ "error": "账号已被禁用" })),
                ));
            }

            crate::modules::auth::record_user_activity(&state.pool, token_data.claims.sub)
                .await
                .map_err(|_| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({ "error": "记录活跃状态失败" })),
                    )
                })?;

            Ok(AuthUser {
                id: token_data.claims.sub,
                role: user.0,
            })
        })
    }
}

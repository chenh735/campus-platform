use axum::extract::{Query, State};
use axum::Json;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::modules::auth::UserInfo;
use crate::modules::materials::MaterialItem;
use crate::modules::projects::{ProjectApplication, ProjectItem};
use crate::modules::reviews::ReviewItem;
use crate::modules::AppState;

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub nickname: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct PageQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct ProfileResponse {
    pub user: UserInfo,
    pub level: i32,
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

pub async fn get_profile(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ProfileResponse>, AppError> {
    let user = sqlx::query_as::<_, UserInfo>(
        "SELECT id, email, nickname, role, experience, status FROM users WHERE id = ?",
    )
    .bind(auth.id)
    .fetch_one(&state.pool)
    .await?;

    let level = calculate_level(user.experience);

    Ok(Json(ProfileResponse { user, level }))
}

pub async fn get_my_materials(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<PageQuery>,
) -> Result<Json<PaginatedResponse<MaterialItem>>, AppError> {
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(10).max(1).min(50);
    let offset = (page - 1) * page_size;

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM materials WHERE user_id = ?")
        .bind(auth.id)
        .fetch_one(&state.pool)
        .await?;

    let items = sqlx::query_as::<_, MaterialItem>(
        "SELECT m.id, m.course_id, m.user_id, u.nickname, c.code as course_code, c.name as course_name, \
         m.title, m.description, m.tag, \
         m.original_name, m.file_size, m.file_type, \
         CASE WHEN m.file_type = 'text/uri-list' THEN 'link' ELSE 'file' END as resource_type, \
         CASE WHEN m.file_type = 'text/uri-list' THEN m.file_path ELSE NULL END as link_url, \
         m.download_count, m.like_count, m.status, m.created_at \
         FROM materials m JOIN users u ON m.user_id = u.id \
         JOIN courses c ON m.course_id = c.id \
         WHERE m.user_id = ? ORDER BY m.created_at DESC LIMIT ? OFFSET ?"
    )
    .bind(auth.id)
    .bind(page_size)
    .bind(offset)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(PaginatedResponse {
        items,
        total,
        page,
        page_size,
    }))
}

pub async fn get_my_reviews(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<PageQuery>,
) -> Result<Json<PaginatedResponse<ReviewItem>>, AppError> {
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(10).max(1).min(50);
    let offset = (page - 1) * page_size;

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM course_reviews WHERE user_id = ?")
        .bind(auth.id)
        .fetch_one(&state.pool)
        .await?;

    let items = sqlx::query_as::<_, ReviewItem>(
        "SELECT r.id, r.course_id, r.user_id, u.nickname, c.code as course_code, c.name as course_name, \
         r.rating, r.difficulty, r.workload, \
         r.content, r.like_count, r.status, r.created_at \
         FROM course_reviews r JOIN users u ON r.user_id = u.id \
         JOIN courses c ON r.course_id = c.id \
         WHERE r.user_id = ? ORDER BY r.created_at DESC LIMIT ? OFFSET ?"
    )
    .bind(auth.id)
    .bind(page_size)
    .bind(offset)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(PaginatedResponse {
        items,
        total,
        page,
        page_size,
    }))
}

pub async fn get_my_projects(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<PageQuery>,
) -> Result<Json<PaginatedResponse<ProjectItem>>, AppError> {
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(10).max(1).min(50);
    let offset = (page - 1) * page_size;

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM projects WHERE user_id = ?")
        .bind(auth.id)
        .fetch_one(&state.pool)
        .await?;

    let items = sqlx::query_as::<_, ProjectItem>(
        "SELECT p.id, p.user_id, u.nickname, p.title, p.type, p.tech_stack, p.description, \
         p.requirements, p.contact, \
         p.required_members, p.current_members, p.deadline, p.status, p.created_at \
         FROM projects p JOIN users u ON p.user_id = u.id \
         WHERE p.user_id = ? ORDER BY p.created_at DESC LIMIT ? OFFSET ?",
    )
    .bind(auth.id)
    .bind(page_size)
    .bind(offset)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(PaginatedResponse {
        items,
        total,
        page,
        page_size,
    }))
}

pub async fn get_my_applications(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<PageQuery>,
) -> Result<Json<PaginatedResponse<ProjectApplication>>, AppError> {
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(10).max(1).min(50);
    let offset = (page - 1) * page_size;

    let total: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM project_applications WHERE user_id = ?")
            .bind(auth.id)
            .fetch_one(&state.pool)
            .await?;

    let items = sqlx::query_as::<_, ProjectApplication>(
        "SELECT pa.id, pa.project_id, p.title as project_title, pa.user_id, u.nickname as applicant_nickname, \
         pa.introduction, pa.contact, pa.status, pa.created_at \
         FROM project_applications pa \
         JOIN projects p ON pa.project_id = p.id \
         JOIN users u ON pa.user_id = u.id \
         WHERE pa.user_id = ? ORDER BY pa.created_at DESC LIMIT ? OFFSET ?"
    )
    .bind(auth.id)
    .bind(page_size)
    .bind(offset)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(PaginatedResponse {
        items,
        total,
        page,
        page_size,
    }))
}

pub async fn update_profile(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<UpdateProfileRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    if let Some(nickname) = &req.nickname {
        if nickname.trim().is_empty() || nickname.len() > 64 {
            return Err(AppError::BadRequest("昵称长度应在1-64个字符".into()));
        }
        let now = Utc::now().naive_utc();
        sqlx::query("UPDATE users SET nickname = ?, updated_at = ? WHERE id = ?")
            .bind(nickname.trim())
            .bind(&now)
            .bind(auth.id)
            .execute(&state.pool)
            .await?;
    }

    Ok(Json(serde_json::json!({ "message": "资料已更新" })))
}

pub async fn change_password(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let current_hash: String = sqlx::query_scalar("SELECT password_hash FROM users WHERE id = ?")
        .bind(auth.id)
        .fetch_one(&state.pool)
        .await?;

    let valid = verify(&req.old_password, &current_hash)
        .map_err(|_| AppError::Internal("密码校验失败".into()))?;

    if !valid {
        return Err(AppError::BadRequest("原密码错误".into()));
    }

    if req.new_password.len() < 6 {
        return Err(AppError::BadRequest("新密码长度至少6位".into()));
    }

    let new_hash = hash(&req.new_password, DEFAULT_COST)
        .map_err(|_| AppError::Internal("密码加密失败".into()))?;

    let now = Utc::now().naive_utc();
    sqlx::query("UPDATE users SET password_hash = ?, updated_at = ? WHERE id = ?")
        .bind(&new_hash)
        .bind(&now)
        .bind(auth.id)
        .execute(&state.pool)
        .await?;

    Ok(Json(serde_json::json!({ "message": "密码已修改" })))
}

pub fn calculate_level(experience: i32) -> i32 {
    match experience {
        0..=9 => 1,
        10..=29 => 2,
        30..=59 => 3,
        60..=99 => 4,
        _ => 5,
    }
}

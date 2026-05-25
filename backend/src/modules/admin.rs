use axum::extract::{Path, Query, State};
use axum::Json;
use chrono::{Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashMap;

use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::modules::auth::UserInfo;
use crate::modules::projects::ProjectItem;
use crate::modules::AppState;

#[derive(Debug, Serialize)]
pub struct DashboardStats {
    pub user_count: i64,
    pub course_count: i64,
    pub material_count: i64,
    pub project_count: i64,
    pub pending_audit_count: i64,
    pub today_active_count: i64,
    pub daily_active_counts: Vec<DailyActiveCount>,
}

#[derive(Debug, Serialize)]
pub struct DailyActiveCount {
    pub date: NaiveDate,
    pub count: i64,
}

pub async fn get_dashboard(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<DashboardStats>, AppError> {
    require_admin(&auth)?;

    let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&state.pool)
        .await?;
    let course_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM courses WHERE status = 'active'")
            .fetch_one(&state.pool)
            .await?;
    let material_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM materials")
        .fetch_one(&state.pool)
        .await?;
    let project_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM projects")
        .fetch_one(&state.pool)
        .await?;
    let pending_audit_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM materials WHERE status = 'pending'")
            .fetch_one(&state.pool)
            .await?;
    let today: NaiveDate = sqlx::query_scalar("SELECT CURRENT_DATE()")
        .fetch_one(&state.pool)
        .await?;
    let activity_rows = sqlx::query_as::<_, (NaiveDate, i64)>(
        "SELECT activity_date, COUNT(*) FROM user_daily_activity \
         WHERE activity_date BETWEEN DATE_SUB(CURRENT_DATE(), INTERVAL 29 DAY) AND CURRENT_DATE() \
         GROUP BY activity_date ORDER BY activity_date ASC",
    )
    .fetch_all(&state.pool)
    .await?;
    let activity_map: HashMap<NaiveDate, i64> = activity_rows.into_iter().collect();
    let daily_active_counts = (0..30)
        .map(|days_ago| {
            let date = today - Duration::days(29 - days_ago);
            DailyActiveCount {
                date,
                count: *activity_map.get(&date).unwrap_or(&0),
            }
        })
        .collect::<Vec<_>>();
    let today_active_count = daily_active_counts
        .last()
        .map(|item| item.count)
        .unwrap_or(0);

    Ok(Json(DashboardStats {
        user_count,
        course_count,
        material_count,
        project_count,
        pending_audit_count,
        today_active_count,
        daily_active_counts,
    }))
}

// Course management
#[derive(Debug, Deserialize)]
pub struct CreateCourseRequest {
    pub code: String,
    pub name: String,
    pub teacher: Option<String>,
    pub credit: Option<f64>,
    pub category: Option<String>,
    pub semester: Option<String>,
    pub description: Option<String>,
}

pub async fn create_course(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateCourseRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    require_admin(&auth)?;

    let now = Utc::now().naive_utc();
    sqlx::query(
        "INSERT INTO courses (code, name, teacher, credit, category, semester, description, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&req.code)
    .bind(&req.name)
    .bind(&req.teacher)
    .bind(req.credit)
    .bind(&req.category)
    .bind(&req.semester)
    .bind(&req.description)
    .bind(&now)
    .bind(&now)
    .execute(&state.pool)
    .await?;

    log_admin_action(
        &state.pool,
        auth.id,
        "create_course",
        "course",
        0,
        &format!("添加课程: {}", req.name),
    )
    .await?;

    Ok(Json(serde_json::json!({ "message": "课程添加成功" })))
}

pub async fn update_course(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    auth: AuthUser,
    Json(req): Json<CreateCourseRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    require_admin(&auth)?;

    let now = Utc::now().naive_utc();
    sqlx::query(
        "UPDATE courses SET code = ?, name = ?, teacher = ?, credit = ?, category = ?, semester = ?, \
         description = ?, updated_at = ? WHERE id = ?"
    )
    .bind(&req.code)
    .bind(&req.name)
    .bind(&req.teacher)
    .bind(req.credit)
    .bind(&req.category)
    .bind(&req.semester)
    .bind(&req.description)
    .bind(&now)
    .bind(id)
    .execute(&state.pool)
    .await?;

    Ok(Json(serde_json::json!({ "message": "课程已更新" })))
}

pub async fn delete_course(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    auth: AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    require_admin(&auth)?;

    sqlx::query("UPDATE courses SET status = 'inactive' WHERE id = ?")
        .bind(id)
        .execute(&state.pool)
        .await?;

    log_admin_action(
        &state.pool,
        auth.id,
        "delete_course",
        "course",
        id,
        "下架课程",
    )
    .await?;

    Ok(Json(serde_json::json!({ "message": "课程已下架" })))
}

// Content audit
#[derive(Debug, Deserialize)]
pub struct AuditListQuery {
    pub r#type: Option<String>,
    pub status: Option<String>,
    pub keyword: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct AuditItem {
    pub id: i64,
    pub title: String,
    pub item_type: String,
    pub author: String,
    pub related: String,
    pub resource_type: String,
    pub link_url: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub status: String,
}

pub async fn list_audit_items(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<AuditListQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    require_admin(&auth)?;

    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(10).max(1).min(50);
    let offset = (page - 1) * page_size;
    let item_type = q.r#type.as_deref().unwrap_or("material");
    let item_status = q.status.as_deref().unwrap_or("approved");
    let keyword = q.keyword.unwrap_or_default();
    let pattern = format!("%{}%", keyword);

    // Content management currently handles learning materials.
    if item_type != "material" {
        return Ok(Json(serde_json::json!({
            "items": [],
            "total": 0,
            "page": page,
            "page_size": page_size
        })));
    }

    // Materials are uploaded as approved by default and can be managed later.
    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM materials m \
         JOIN users u ON m.user_id = u.id \
         JOIN courses c ON m.course_id = c.id \
         WHERE m.status = ? AND (m.title LIKE ? OR u.nickname LIKE ? OR c.name LIKE ? OR c.code LIKE ?)"
    )
    .bind(item_status)
    .bind(&pattern)
    .bind(&pattern)
    .bind(&pattern)
    .bind(&pattern)
    .fetch_one(&state.pool).await?;

    let items = sqlx::query_as::<_, AuditItem>(
        "SELECT m.id, m.title, 'material' as item_type, u.nickname as author, c.name as related, \
         CASE WHEN m.file_type = 'text/uri-list' THEN 'link' ELSE 'file' END as resource_type, \
         CASE WHEN m.file_type = 'text/uri-list' THEN m.file_path ELSE NULL END as link_url, \
         m.created_at, m.status \
         FROM materials m \
         JOIN users u ON m.user_id = u.id \
         JOIN courses c ON m.course_id = c.id \
         WHERE m.status = ? AND (m.title LIKE ? OR u.nickname LIKE ? OR c.name LIKE ? OR c.code LIKE ?) \
         ORDER BY m.created_at ASC LIMIT ? OFFSET ?"
    )
    .bind(item_status)
    .bind(&pattern)
    .bind(&pattern)
    .bind(&pattern)
    .bind(&pattern)
    .bind(page_size)
    .bind(offset)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(serde_json::json!({
        "items": items,
        "total": total,
        "page": page,
        "page_size": page_size
    })))
}

#[derive(Debug, Deserialize)]
pub struct AuditActionRequest {
    pub status: String,
}

pub async fn audit_material(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    auth: AuthUser,
    Json(req): Json<AuditActionRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    require_admin(&auth)?;

    let valid_statuses = ["approved", "rejected", "hidden"];
    if !valid_statuses.contains(&req.status.as_str()) {
        return Err(AppError::BadRequest("无效的状态".into()));
    }

    sqlx::query("UPDATE materials SET status = ? WHERE id = ?")
        .bind(&req.status)
        .bind(id)
        .execute(&state.pool)
        .await?;

    log_admin_action(
        &state.pool,
        auth.id,
        "update_material_status",
        "material",
        id,
        &format!("修改资料状态: {}", req.status),
    )
    .await?;

    Ok(Json(serde_json::json!({ "message": "资料状态已更新" })))
}

pub async fn update_review_status(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    auth: AuthUser,
    Json(req): Json<AuditActionRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    require_admin(&auth)?;

    sqlx::query("UPDATE course_reviews SET status = ? WHERE id = ?")
        .bind(&req.status)
        .bind(id)
        .execute(&state.pool)
        .await?;

    Ok(Json(serde_json::json!({ "message": "评价状态已更新" })))
}

// User management
#[derive(Debug, Deserialize)]
pub struct UserListQuery {
    pub keyword: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

pub async fn list_users(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<UserListQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    require_admin(&auth)?;

    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(10).max(1).min(50);
    let offset = (page - 1) * page_size;
    let keyword = q.keyword.unwrap_or_default();

    let users = sqlx::query_as::<_, UserInfo>(
        "SELECT id, email, nickname, role, experience, status FROM users \
         WHERE email LIKE ? OR nickname LIKE ? \
         ORDER BY id ASC LIMIT ? OFFSET ?",
    )
    .bind(format!("%{}%", keyword))
    .bind(format!("%{}%", keyword))
    .bind(page_size)
    .bind(offset)
    .fetch_all(&state.pool)
    .await?;

    let total: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE email LIKE ? OR nickname LIKE ?")
            .bind(format!("%{}%", keyword))
            .bind(format!("%{}%", keyword))
            .fetch_one(&state.pool)
            .await?;

    Ok(Json(serde_json::json!({
        "users": users,
        "total": total,
        "page": page,
        "page_size": page_size
    })))
}

pub async fn update_user_status(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    auth: AuthUser,
    Json(req): Json<AuditActionRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    require_admin(&auth)?;

    let valid = ["active", "disabled"];
    if !valid.contains(&req.status.as_str()) {
        return Err(AppError::BadRequest("无效的状态".into()));
    }

    sqlx::query("UPDATE users SET status = ? WHERE id = ?")
        .bind(&req.status)
        .bind(id)
        .execute(&state.pool)
        .await?;

    log_admin_action(
        &state.pool,
        auth.id,
        "update_user_status",
        "user",
        id,
        &format!("用户状态: {}", req.status),
    )
    .await?;

    Ok(Json(serde_json::json!({ "message": "用户状态已更新" })))
}

// Project management
#[derive(Debug, Deserialize)]
pub struct AdminProjectListQuery {
    pub keyword: Option<String>,
    pub r#type: Option<String>,
    pub status: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

pub async fn list_projects(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<AdminProjectListQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    require_admin(&auth)?;

    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(10).max(1).min(50);
    let offset = (page - 1) * page_size;
    let keyword = q.keyword.unwrap_or_default();
    let pattern = format!("%{}%", keyword);
    let ptype = q.r#type.unwrap_or_default();
    let pstatus = q.status.unwrap_or_default();

    let projects = sqlx::query_as::<_, ProjectItem>(
        "SELECT p.id, p.user_id, u.nickname, p.title, p.type, p.tech_stack, p.description, \
         p.requirements, p.contact, p.required_members, p.current_members, p.deadline, p.status, p.created_at \
         FROM projects p JOIN users u ON p.user_id = u.id \
         WHERE (p.title LIKE ? OR p.tech_stack LIKE ? OR u.nickname LIKE ?) \
         AND (p.type = ? OR ? = '') AND (p.status = ? OR ? = '') \
         ORDER BY p.created_at DESC LIMIT ? OFFSET ?",
    )
    .bind(&pattern)
    .bind(&pattern)
    .bind(&pattern)
    .bind(&ptype)
    .bind(&ptype)
    .bind(&pstatus)
    .bind(&pstatus)
    .bind(page_size)
    .bind(offset)
    .fetch_all(&state.pool)
    .await?;

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM projects p JOIN users u ON p.user_id = u.id \
         WHERE (p.title LIKE ? OR p.tech_stack LIKE ? OR u.nickname LIKE ?) \
         AND (p.type = ? OR ? = '') AND (p.status = ? OR ? = '')",
    )
    .bind(&pattern)
    .bind(&pattern)
    .bind(&pattern)
    .bind(&ptype)
    .bind(&ptype)
    .bind(&pstatus)
    .bind(&pstatus)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(serde_json::json!({
        "projects": projects,
        "total": total,
        "page": page,
        "page_size": page_size
    })))
}

pub async fn update_project_status(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    auth: AuthUser,
    Json(req): Json<AuditActionRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    require_admin(&auth)?;

    if !["hidden", "closed"].contains(&req.status.as_str()) {
        return Err(AppError::BadRequest("无效的项目状态".into()));
    }

    sqlx::query("UPDATE projects SET status = ? WHERE id = ?")
        .bind(&req.status)
        .bind(id)
        .execute(&state.pool)
        .await?;

    Ok(Json(serde_json::json!({ "message": "项目状态已更新" })))
}

// Admin logs
#[derive(Debug, Serialize, FromRow)]
pub struct AdminLog {
    pub id: i64,
    pub admin_id: i64,
    pub action: String,
    pub target_type: String,
    pub target_id: Option<i64>,
    pub detail: Option<String>,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct LogQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

pub async fn get_admin_logs(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<LogQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    require_admin(&auth)?;

    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(10).max(1).min(50);
    let offset = (page - 1) * page_size;

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM admin_logs")
        .fetch_one(&state.pool)
        .await?;

    let logs = sqlx::query_as::<_, AdminLog>(
        "SELECT id, admin_id, action, target_type, target_id, detail, created_at \
         FROM admin_logs ORDER BY created_at DESC LIMIT ? OFFSET ?",
    )
    .bind(page_size)
    .bind(offset)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(serde_json::json!({
        "logs": logs,
        "total": total,
        "page": page,
        "page_size": page_size
    })))
}

fn require_admin(auth: &AuthUser) -> Result<(), AppError> {
    if auth.role != "admin" {
        return Err(AppError::Forbidden("需要管理员权限".into()));
    }
    Ok(())
}

async fn log_admin_action(
    pool: &sqlx::MySqlPool,
    admin_id: i64,
    action: &str,
    target_type: &str,
    target_id: i64,
    detail: &str,
) -> Result<(), AppError> {
    let now = Utc::now().naive_utc();
    sqlx::query(
        "INSERT INTO admin_logs (admin_id, action, target_type, target_id, detail, created_at) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(admin_id)
    .bind(action)
    .bind(target_type)
    .bind(target_id)
    .bind(detail)
    .bind(&now)
    .execute(pool)
    .await?;
    Ok(())
}

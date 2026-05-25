use axum::extract::{Path, Query, State};
use axum::Json;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::modules::AppState;

#[derive(Debug, Deserialize)]
pub struct ProjectListQuery {
    pub keyword: Option<String>,
    pub r#type: Option<String>,
    pub status: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ProjectItem {
    pub id: i64,
    pub user_id: i64,
    pub nickname: String,
    pub title: String,
    pub r#type: String,
    pub tech_stack: Option<String>,
    pub description: String,
    pub requirements: Option<String>,
    pub contact: Option<String>,
    pub required_members: i32,
    pub current_members: i32,
    pub deadline: Option<chrono::NaiveDate>,
    pub status: String,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub title: String,
    pub r#type: String,
    pub tech_stack: Option<String>,
    pub description: String,
    pub requirements: Option<String>,
    pub required_members: i32,
    pub contact: String,
    pub deadline: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ApplyRequest {
    pub introduction: String,
    pub contact: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProjectStatusRequest {
    pub status: String,
}

pub async fn list_projects(
    State(state): State<AppState>,
    Query(q): Query<ProjectListQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(10).max(1).min(50);
    let offset = (page - 1) * page_size;
    let keyword = q.keyword.unwrap_or_default();
    let ptype = q.r#type.unwrap_or_default();
    let pstatus = q.status.unwrap_or_else(|| "recruiting".into());

    let projects = sqlx::query_as::<_, ProjectItem>(
        "SELECT p.id, p.user_id, u.nickname, p.title, p.type, p.tech_stack, p.description, \
         p.requirements, p.contact, \
         p.required_members, p.current_members, p.deadline, p.status, p.created_at \
         FROM projects p JOIN users u ON p.user_id = u.id \
         WHERE p.status NOT IN ('hidden') \
         AND (p.title LIKE ? OR p.tech_stack LIKE ?) \
         AND (p.type = ? OR ? = '') \
         AND (p.status = ? OR ? = '') \
         ORDER BY p.created_at DESC LIMIT ? OFFSET ?",
    )
    .bind(format!("%{}%", keyword))
    .bind(format!("%{}%", keyword))
    .bind(&ptype)
    .bind(&ptype)
    .bind(&pstatus)
    .bind(&pstatus)
    .bind(page_size)
    .bind(offset)
    .fetch_all(&state.pool)
    .await?;

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM projects p WHERE p.status NOT IN ('hidden') \
         AND (p.title LIKE ? OR p.tech_stack LIKE ?) \
         AND (p.type = ? OR ? = '') \
         AND (p.status = ? OR ? = '')",
    )
    .bind(format!("%{}%", keyword))
    .bind(format!("%{}%", keyword))
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

pub async fn get_project(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<ProjectItem>, AppError> {
    let project = sqlx::query_as::<_, ProjectItem>(
        "SELECT p.id, p.user_id, u.nickname, p.title, p.type, p.tech_stack, p.description, \
         p.requirements, p.contact, \
         p.required_members, p.current_members, p.deadline, p.status, p.created_at \
         FROM projects p JOIN users u ON p.user_id = u.id WHERE p.id = ? AND p.status <> 'hidden'",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound("项目不存在".into()))?;

    Ok(Json(project))
}

pub async fn create_project(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateProjectRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    if req.title.trim().is_empty() {
        return Err(AppError::BadRequest("标题不能为空".into()));
    }

    let valid_types = ["course_project", "competition", "research", "personal"];
    if !valid_types.contains(&req.r#type.as_str()) {
        return Err(AppError::BadRequest("无效的项目类型".into()));
    }
    if req.required_members < 1 || req.required_members > 20 {
        return Err(AppError::BadRequest("需要人数范围为1-20".into()));
    }

    let mut tx = state.pool.begin().await?;
    let _: i64 = sqlx::query_scalar("SELECT id FROM users WHERE id = ? FOR UPDATE")
        .bind(auth.id)
        .fetch_one(&mut *tx)
        .await?;
    let active_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM projects WHERE user_id = ? AND status NOT IN ('closed', 'hidden')",
    )
    .bind(auth.id)
    .fetch_one(&mut *tx)
    .await?;
    if active_count >= 5 {
        return Err(AppError::BadRequest(
            "每位用户最多发布5个未关闭或未隐藏的招募".into(),
        ));
    }

    let now = Utc::now().naive_utc();
    let deadline = req
        .deadline
        .and_then(|d| chrono::NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok());
    let status = if req.required_members == 1 {
        "full"
    } else {
        "recruiting"
    };

    sqlx::query(
        "INSERT INTO projects (user_id, title, type, tech_stack, description, requirements, \
         required_members, contact, deadline, status, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(auth.id)
    .bind(req.title.trim())
    .bind(&req.r#type)
    .bind(&req.tech_stack)
    .bind(req.description.trim())
    .bind(&req.requirements)
    .bind(req.required_members)
    .bind(&req.contact)
    .bind(deadline)
    .bind(status)
    .bind(&now)
    .bind(&now)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    Ok(Json(serde_json::json!({ "message": "项目发布成功" })))
}

pub async fn update_project(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    auth: AuthUser,
    Json(req): Json<CreateProjectRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let valid_types = ["course_project", "competition", "research", "personal"];
    if !valid_types.contains(&req.r#type.as_str()) {
        return Err(AppError::BadRequest("无效的项目类型".into()));
    }
    if !(1..=20).contains(&req.required_members) {
        return Err(AppError::BadRequest("招募人数应为1至20人".into()));
    }

    let project: (i64, i32, String) =
        sqlx::query_as("SELECT user_id, current_members, status FROM projects WHERE id = ?")
            .bind(id)
            .fetch_optional(&state.pool)
            .await?
            .ok_or(AppError::NotFound("项目不存在".into()))?;

    if project.0 != auth.id && auth.role != "admin" {
        return Err(AppError::Forbidden("无权编辑此项目".into()));
    }
    if req.required_members < project.1 {
        return Err(AppError::BadRequest("招募人数不能少于当前成员数".into()));
    }

    let now = Utc::now().naive_utc();
    let deadline = req
        .deadline
        .and_then(|d| chrono::NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok());
    let status = if ["closed", "hidden"].contains(&project.2.as_str()) {
        project.2
    } else if project.1 >= req.required_members {
        String::from("full")
    } else {
        String::from("recruiting")
    };

    sqlx::query(
        "UPDATE projects SET title = ?, type = ?, tech_stack = ?, description = ?, requirements = ?, \
         required_members = ?, contact = ?, deadline = ?, status = ?, updated_at = ? WHERE id = ?"
    )
    .bind(req.title.trim())
    .bind(&req.r#type)
    .bind(&req.tech_stack)
    .bind(req.description.trim())
    .bind(&req.requirements)
    .bind(req.required_members)
    .bind(&req.contact)
    .bind(deadline)
    .bind(status)
    .bind(&now)
    .bind(id)
    .execute(&state.pool)
    .await?;

    Ok(Json(serde_json::json!({ "message": "项目已更新" })))
}

pub async fn delete_project(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    auth: AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    let owner: i64 = sqlx::query_scalar("SELECT user_id FROM projects WHERE id = ?")
        .bind(id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(AppError::NotFound("项目不存在".into()))?;

    if owner != auth.id && auth.role != "admin" {
        return Err(AppError::Forbidden("无权删除此项目".into()));
    }

    sqlx::query("UPDATE projects SET status = 'closed' WHERE id = ?")
        .bind(id)
        .execute(&state.pool)
        .await?;

    Ok(Json(serde_json::json!({ "message": "项目已关闭" })))
}

pub async fn update_project_status(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    auth: AuthUser,
    Json(req): Json<UpdateProjectStatusRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    if !["closed", "hidden"].contains(&req.status.as_str()) {
        return Err(AppError::BadRequest("只能关闭或隐藏招募".into()));
    }

    let owner: i64 = sqlx::query_scalar("SELECT user_id FROM projects WHERE id = ?")
        .bind(id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(AppError::NotFound("项目不存在".into()))?;

    if owner != auth.id {
        return Err(AppError::Forbidden("只有发布者可以处理该招募".into()));
    }

    sqlx::query("UPDATE projects SET status = ?, updated_at = ? WHERE id = ?")
        .bind(&req.status)
        .bind(Utc::now().naive_utc())
        .bind(id)
        .execute(&state.pool)
        .await?;

    Ok(Json(serde_json::json!({ "message": "招募状态已更新" })))
}

pub async fn apply_project(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    auth: AuthUser,
    Json(req): Json<ApplyRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let project = sqlx::query_as::<_, (i64, String, i32, i32)>(
        "SELECT user_id, status, required_members, current_members FROM projects WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound("项目不存在".into()))?;

    if project.1 != "recruiting" {
        return Err(AppError::BadRequest("该项目不在招募中".into()));
    }

    if project.0 == auth.id {
        return Err(AppError::BadRequest("不能申请自己的项目".into()));
    }

    let now = Utc::now().naive_utc();

    let result = sqlx::query(
        "INSERT INTO project_applications (project_id, user_id, introduction, contact, status, created_at, updated_at) \
         VALUES (?, ?, ?, ?, 'pending', ?, ?)"
    )
    .bind(id)
    .bind(auth.id)
    .bind(req.introduction.trim())
    .bind(req.contact.trim())
    .bind(&now)
    .bind(&now)
    .execute(&state.pool)
    .await;

    match result {
        Ok(_) => Ok(Json(serde_json::json!({ "message": "申请已提交" }))),
        Err(e) => {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.message().contains("Duplicate") {
                    return Err(AppError::Conflict("已申请过该项目".into()));
                }
            }
            Err(AppError::Internal(format!("申请失败: {}", e)))
        }
    }
}

#[derive(Debug, Serialize, FromRow)]
pub struct ProjectApplication {
    pub id: i64,
    pub project_id: i64,
    pub project_title: String,
    pub user_id: i64,
    pub applicant_nickname: String,
    pub introduction: String,
    pub contact: String,
    pub status: String,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct ApplicationListQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct HandleApplicationRequest {
    pub status: String,
}

pub async fn list_received_applications(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<ApplicationListQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(10).max(1).min(50);
    let offset = (page - 1) * page_size;

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM project_applications pa \
         JOIN projects p ON pa.project_id = p.id WHERE p.user_id = ?",
    )
    .bind(auth.id)
    .fetch_one(&state.pool)
    .await?;

    let items = sqlx::query_as::<_, ProjectApplication>(
        "SELECT pa.id, pa.project_id, p.title as project_title, pa.user_id, u.nickname as applicant_nickname, \
         pa.introduction, pa.contact, pa.status, pa.created_at \
         FROM project_applications pa \
         JOIN projects p ON pa.project_id = p.id \
         JOIN users u ON pa.user_id = u.id \
         WHERE p.user_id = ? ORDER BY pa.created_at DESC LIMIT ? OFFSET ?",
    )
    .bind(auth.id)
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

pub async fn handle_application(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    auth: AuthUser,
    Json(req): Json<HandleApplicationRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    if !["accepted", "rejected"].contains(&req.status.as_str()) {
        return Err(AppError::BadRequest("无效的处理状态".into()));
    }

    let mut tx = state.pool.begin().await?;
    let application = sqlx::query_as::<_, (i64, i64, String, String, i32, i32)>(
        "SELECT pa.project_id, p.user_id, pa.status, p.status, p.current_members, p.required_members \
         FROM project_applications pa JOIN projects p ON pa.project_id = p.id \
         WHERE pa.id = ? FOR UPDATE",
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(AppError::NotFound("申请不存在".into()))?;

    if application.1 != auth.id {
        return Err(AppError::Forbidden("只有项目发布者可以处理申请".into()));
    }
    if application.2 != "pending" {
        return Err(AppError::BadRequest("该申请已处理".into()));
    }

    let now = Utc::now().naive_utc();
    if req.status == "accepted" {
        if application.3 != "recruiting" || application.4 >= application.5 {
            return Err(AppError::BadRequest("项目当前无法接收新成员".into()));
        }

        sqlx::query(
            "UPDATE projects SET current_members = current_members + 1, \
             status = CASE WHEN current_members + 1 >= required_members THEN 'full' ELSE status END, \
             updated_at = ? WHERE id = ?",
        )
        .bind(&now)
        .bind(application.0)
        .execute(&mut *tx)
        .await?;
    }

    sqlx::query("UPDATE project_applications SET status = ?, updated_at = ? WHERE id = ?")
        .bind(&req.status)
        .bind(&now)
        .bind(id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(Json(serde_json::json!({ "message": "申请已处理" })))
}

pub async fn delete_application(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    auth: AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    let app = sqlx::query_as::<_, (i64, String)>(
        "SELECT user_id, status FROM project_applications WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound("申请不存在".into()))?;

    if app.0 != auth.id && auth.role != "admin" {
        return Err(AppError::Forbidden("无权删除此申请".into()));
    }
    if app.1 == "accepted" {
        return Err(AppError::BadRequest("已通过的申请不能删除".into()));
    }

    sqlx::query("DELETE FROM project_applications WHERE id = ?")
        .bind(id)
        .execute(&state.pool)
        .await?;

    Ok(Json(serde_json::json!({ "message": "申请已删除" })))
}

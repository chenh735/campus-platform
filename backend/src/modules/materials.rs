use axum::extract::{Path, Query, State};
use axum::http::header;
use axum::{body::Body, response::IntoResponse, Json};
use chrono::Utc;
use serde::Serialize;
use sqlx::FromRow;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::modules::AppState;

#[derive(Debug, serde::Deserialize)]
pub struct PageQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Serialize, FromRow)]
pub struct MaterialItem {
    pub id: i64,
    pub course_id: i64,
    pub user_id: i64,
    pub nickname: String,
    pub course_code: Option<String>,
    pub course_name: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub tag: Option<String>,
    pub original_name: String,
    pub file_size: i64,
    pub file_type: Option<String>,
    pub resource_type: String,
    pub link_url: Option<String>,
    pub download_count: i32,
    pub like_count: i32,
    pub status: String,
    pub created_at: chrono::NaiveDateTime,
}

pub async fn list_materials(
    State(state): State<AppState>,
    Path(course_id): Path<i64>,
    Query(q): Query<PageQuery>,
) -> Result<Json<PaginatedResponse<MaterialItem>>, AppError> {
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(10).max(1).min(50);
    let offset = (page - 1) * page_size;

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM materials WHERE course_id = ? AND status = 'approved'",
    )
    .bind(course_id)
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
         WHERE m.course_id = ? AND m.status = 'approved' ORDER BY m.like_count DESC, m.created_at DESC \
         LIMIT ? OFFSET ?"
    )
    .bind(course_id)
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

pub async fn upload_material(
    State(state): State<AppState>,
    Path(course_id): Path<i64>,
    auth: AuthUser,
    mut multipart: axum::extract::Multipart,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut title = String::new();
    let mut description = String::new();
    let mut tag = String::new();
    let mut file_data: Vec<u8> = Vec::new();
    let mut original_name = String::new();
    let mut file_type = String::new();
    let mut resource_type = String::from("file");
    let mut link_url = String::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("解析上传数据失败: {}", e)))?
    {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "title" => title = field.text().await.unwrap_or_default(),
            "description" => description = field.text().await.unwrap_or_default(),
            "tag" => tag = field.text().await.unwrap_or_default(),
            "resource_type" => resource_type = field.text().await.unwrap_or_default(),
            "link_url" => link_url = field.text().await.unwrap_or_default(),
            "file" => {
                original_name = field.file_name().unwrap_or("unknown").to_string();
                file_type = field
                    .content_type()
                    .unwrap_or("application/octet-stream")
                    .to_string();
                file_data = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::BadRequest(format!("读取文件失败: {}", e)))?
                    .to_vec();
            }
            _ => {}
        }
    }

    if title.trim().is_empty() {
        return Err(AppError::BadRequest("资料标题不能为空".into()));
    }

    if resource_type != "file" && resource_type != "link" {
        return Err(AppError::BadRequest("无效的资料类型".into()));
    }

    if resource_type == "file" && file_data.is_empty() {
        return Err(AppError::BadRequest("请选择文件".into()));
    }
    if resource_type == "link" && !is_valid_link(&link_url) {
        return Err(AppError::BadRequest(
            "请输入有效的 http 或 https 链接".into(),
        ));
    }

    let max_size = state.config.max_upload_mb * 1024 * 1024;
    if resource_type == "file" && file_data.len() as u64 > max_size {
        return Err(AppError::BadRequest(format!(
            "文件大小超过 {}MB 限制",
            state.config.max_upload_mb
        )));
    }

    if resource_type == "file" {
        let ext = std::path::Path::new(&original_name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        let allowed = [
            "pdf", "docx", "ppt", "pptx", "zip", "jpg", "jpeg", "png", "gif",
        ];
        if !allowed.contains(&ext.as_str()) {
            return Err(AppError::BadRequest("不支持的文件类型".into()));
        }
    }

    let now = Utc::now();
    let mut tx = state.pool.begin().await?;

    let _: i64 = sqlx::query_scalar("SELECT id FROM users WHERE id = ? FOR UPDATE")
        .bind(auth.id)
        .fetch_one(&mut *tx)
        .await?;
    let used_size: i64 = sqlx::query_scalar(
        "SELECT CAST(COALESCE(SUM(file_size), 0) AS SIGNED) FROM materials \
         WHERE user_id = ? AND status <> 'deleted'",
    )
    .bind(auth.id)
    .fetch_one(&mut *tx)
    .await?;
    const TOTAL_UPLOAD_LIMIT: i64 = 1024 * 1024 * 1024;
    if resource_type == "file" && used_size + file_data.len() as i64 > TOTAL_UPLOAD_LIMIT {
        return Err(AppError::BadRequest(
            "已超过个人资料总容量限制（1 GiB）".into(),
        ));
    }

    let (storage_path, stored_name, stored_type, stored_size, created_file) =
        if resource_type == "link" {
            (
                link_url.trim().to_string(),
                String::from("外部链接"),
                String::from("text/uri-list"),
                0_i64,
                None,
            )
        } else {
            let relative_path = format!(
                "materials/{}/{}/{}_{}",
                now.format("%Y"),
                now.format("%m"),
                Uuid::new_v4(),
                &original_name
            );
            let full_path = PathBuf::from(&state.config.upload_dir).join(&relative_path);
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent)
                    .await
                    .map_err(|e| AppError::Internal(format!("创建目录失败: {}", e)))?;
            }
            let mut f = tokio::fs::File::create(&full_path)
                .await
                .map_err(|e| AppError::Internal(format!("创建文件失败: {}", e)))?;
            f.write_all(&file_data)
                .await
                .map_err(|e| AppError::Internal(format!("写入文件失败: {}", e)))?;
            (
                relative_path,
                original_name,
                file_type,
                file_data.len() as i64,
                Some(full_path),
            )
        };
    let created_at = now.naive_utc();

    let insert_result = sqlx::query(
        "INSERT INTO materials (course_id, user_id, title, description, tag, original_name, file_path, \
         file_size, file_type, status, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 'approved', ?, ?)"
    )
    .bind(course_id)
    .bind(auth.id)
    .bind(title.trim())
    .bind(&description)
    .bind(&tag)
    .bind(&stored_name)
    .bind(&storage_path)
    .bind(stored_size)
    .bind(&stored_type)
    .bind(&created_at)
    .bind(&created_at)
    .execute(&mut *tx)
    .await;
    if let Err(err) = insert_result {
        if let Some(path) = &created_file {
            let _ = fs::remove_file(path).await;
        }
        return Err(err.into());
    }
    if let Err(err) = tx.commit().await {
        if let Some(path) = &created_file {
            let _ = fs::remove_file(path).await;
        }
        return Err(err.into());
    }

    Ok(Json(serde_json::json!({ "message": "资料上传成功" })))
}

pub async fn download_material(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    _auth: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let material = sqlx::query_as::<_, (String, String, Option<String>)>(
        "SELECT file_path, original_name, file_type FROM materials WHERE id = ? AND status = 'approved'"
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound("资料不存在或不可下载".into()))?;

    if material.2.as_deref() == Some("text/uri-list") {
        return Err(AppError::BadRequest("链接资料请直接访问链接".into()));
    }

    let full_path = PathBuf::from(&state.config.upload_dir).join(&material.0);

    let data = tokio::fs::read(&full_path)
        .await
        .map_err(|_| AppError::NotFound("文件不存在".into()))?;

    sqlx::query("UPDATE materials SET download_count = download_count + 1 WHERE id = ?")
        .bind(id)
        .execute(&state.pool)
        .await?;

    let content_type = material
        .2
        .unwrap_or_else(|| "application/octet-stream".into());

    let response = axum::response::Response::builder()
        .header(header::CONTENT_TYPE, &content_type)
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", material.1),
        )
        .body(Body::from(data))
        .unwrap();

    Ok(response)
}

fn is_valid_link(link: &str) -> bool {
    let link = link.trim();
    (link.starts_with("https://") || link.starts_with("http://"))
        && !link.chars().any(char::is_whitespace)
        && link.len() <= 512
}

pub async fn like_material(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    auth: AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    let material =
        sqlx::query_as::<_, (i64, String)>("SELECT user_id, status FROM materials WHERE id = ?")
            .bind(id)
            .fetch_optional(&state.pool)
            .await?
            .ok_or(AppError::NotFound("资料不存在".into()))?;

    if material.1 != "approved" {
        return Err(AppError::BadRequest("该资料不可见".into()));
    }

    let now = Utc::now().naive_utc();

    let result = sqlx::query(
        "INSERT INTO material_likes (material_id, user_id, created_at) VALUES (?, ?, ?)",
    )
    .bind(id)
    .bind(auth.id)
    .bind(&now)
    .execute(&state.pool)
    .await;

    match result {
        Ok(_) => {
            sqlx::query("UPDATE materials SET like_count = like_count + 1 WHERE id = ?")
                .bind(id)
                .execute(&state.pool)
                .await?;

            sqlx::query("UPDATE users SET experience = experience + 1 WHERE id = ?")
                .bind(material.0)
                .execute(&state.pool)
                .await?;

            Ok(Json(serde_json::json!({ "message": "点赞成功" })))
        }
        Err(e) => {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.message().contains("Duplicate") {
                    return Err(AppError::Conflict("已点赞过".into()));
                }
            }
            Err(AppError::Internal(format!("点赞失败: {}", e)))
        }
    }
}

pub async fn unlike_material(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    auth: AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    let material = sqlx::query_as::<_, (i64,)>("SELECT user_id FROM materials WHERE id = ?")
        .bind(id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(AppError::NotFound("资料不存在".into()))?;

    let deleted = sqlx::query("DELETE FROM material_likes WHERE material_id = ? AND user_id = ?")
        .bind(id)
        .bind(auth.id)
        .execute(&state.pool)
        .await?
        .rows_affected();

    if deleted > 0 {
        sqlx::query("UPDATE materials SET like_count = GREATEST(like_count - 1, 0) WHERE id = ?")
            .bind(id)
            .execute(&state.pool)
            .await?;

        sqlx::query("UPDATE users SET experience = GREATEST(experience - 1, 0) WHERE id = ?")
            .bind(material.0)
            .execute(&state.pool)
            .await?;
    }

    Ok(Json(serde_json::json!({ "message": "已取消点赞" })))
}

pub async fn delete_material(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    auth: AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    let material = sqlx::query_as::<_, (i64, String, Option<String>)>(
        "SELECT user_id, file_path, file_type FROM materials WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound("资料不存在".into()))?;

    if material.0 != auth.id && auth.role != "admin" {
        return Err(AppError::Forbidden("无权删除此资料".into()));
    }

    sqlx::query("UPDATE materials SET status = 'deleted' WHERE id = ?")
        .bind(id)
        .execute(&state.pool)
        .await?;

    if material.2.as_deref() != Some("text/uri-list") {
        let full_path = PathBuf::from(&state.config.upload_dir).join(&material.1);
        let _ = fs::remove_file(full_path).await;
    }

    Ok(Json(serde_json::json!({ "message": "资料已删除" })))
}

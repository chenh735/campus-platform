use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::modules::AppState;

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Serialize)]
pub struct CourseLikeStatus {
    pub review_ids: Vec<i64>,
    pub material_ids: Vec<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateReviewRequest {
    pub rating: i32,
    pub difficulty: Option<String>,
    pub workload: Option<String>,
    pub content: String,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ReviewItem {
    pub id: i64,
    pub course_id: i64,
    pub user_id: i64,
    pub nickname: String,
    pub course_code: Option<String>,
    pub course_name: Option<String>,
    pub rating: i32,
    pub difficulty: Option<String>,
    pub workload: Option<String>,
    pub content: String,
    pub like_count: i32,
    pub status: String,
    pub created_at: chrono::NaiveDateTime,
}

pub async fn list_reviews(
    State(state): State<AppState>,
    Path(course_id): Path<i64>,
    Query(q): Query<PageQuery>,
) -> Result<Json<PaginatedResponse<ReviewItem>>, AppError> {
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(10).max(1).min(50);
    let offset = (page - 1) * page_size;

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM course_reviews WHERE course_id = ? AND status = 'visible'",
    )
    .bind(course_id)
    .fetch_one(&state.pool)
    .await?;

    let items = sqlx::query_as::<_, ReviewItem>(
        "SELECT r.id, r.course_id, r.user_id, u.nickname, c.code as course_code, c.name as course_name, \
         r.rating, r.difficulty, r.workload, r.content, \
         r.like_count, r.status, r.created_at \
         FROM course_reviews r JOIN users u ON r.user_id = u.id \
         JOIN courses c ON r.course_id = c.id \
         WHERE r.course_id = ? AND r.status = 'visible' ORDER BY r.like_count DESC, r.created_at DESC \
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

pub async fn get_course_like_status(
    State(state): State<AppState>,
    Path(course_id): Path<i64>,
    auth: AuthUser,
) -> Result<Json<CourseLikeStatus>, AppError> {
    let review_ids = sqlx::query_scalar(
        "SELECT rl.review_id FROM review_likes rl \
         JOIN course_reviews r ON rl.review_id = r.id \
         WHERE rl.user_id = ? AND r.course_id = ? AND r.status = 'visible'",
    )
    .bind(auth.id)
    .bind(course_id)
    .fetch_all(&state.pool)
    .await?;

    let material_ids = sqlx::query_scalar(
        "SELECT ml.material_id FROM material_likes ml \
         JOIN materials m ON ml.material_id = m.id \
         WHERE ml.user_id = ? AND m.course_id = ? AND m.status = 'approved'",
    )
    .bind(auth.id)
    .bind(course_id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(CourseLikeStatus {
        review_ids,
        material_ids,
    }))
}

pub async fn create_review(
    State(state): State<AppState>,
    Path(course_id): Path<i64>,
    auth: AuthUser,
    Json(req): Json<CreateReviewRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    if req.rating < 1 || req.rating > 5 {
        return Err(AppError::BadRequest("评分范围为1-5".into()));
    }

    if req.content.trim().is_empty() {
        return Err(AppError::BadRequest("评价内容不能为空".into()));
    }

    let now = Utc::now().naive_utc();

    let result = sqlx::query(
        "INSERT INTO course_reviews (course_id, user_id, rating, difficulty, workload, content, status, like_count, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?, 'visible', 0, ?, ?) \
         ON DUPLICATE KEY UPDATE \
         rating = VALUES(rating), difficulty = VALUES(difficulty), workload = VALUES(workload), \
         content = VALUES(content), status = 'visible', updated_at = VALUES(updated_at)"
    )
    .bind(course_id)
    .bind(auth.id)
    .bind(req.rating)
    .bind(&req.difficulty)
    .bind(&req.workload)
    .bind(req.content.trim())
    .bind(&now)
    .bind(&now)
    .execute(&state.pool)
    .await;

    match result {
        Ok(_) => Ok(Json(serde_json::json!({ "message": "评价发表成功" }))),
        Err(e) => Err(AppError::Internal(format!("发表评价失败: {}", e))),
    }
}

pub async fn update_review(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    auth: AuthUser,
    Json(req): Json<CreateReviewRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let review = sqlx::query_as::<_, (i64,)>("SELECT user_id FROM course_reviews WHERE id = ?")
        .bind(id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(AppError::NotFound("评价不存在".into()))?;

    if review.0 != auth.id && auth.role != "admin" {
        return Err(AppError::Forbidden("无权编辑此评价".into()));
    }

    let now = Utc::now().naive_utc();

    sqlx::query(
        "UPDATE course_reviews SET rating = ?, difficulty = ?, workload = ?, content = ?, updated_at = ? WHERE id = ?"
    )
    .bind(req.rating)
    .bind(&req.difficulty)
    .bind(&req.workload)
    .bind(req.content.trim())
    .bind(&now)
    .bind(id)
    .execute(&state.pool)
    .await?;

    Ok(Json(serde_json::json!({ "message": "评价已更新" })))
}

pub async fn delete_review(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    auth: AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    let review = sqlx::query_as::<_, (i64,)>("SELECT user_id FROM course_reviews WHERE id = ?")
        .bind(id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(AppError::NotFound("评价不存在".into()))?;

    if review.0 != auth.id && auth.role != "admin" {
        return Err(AppError::Forbidden("无权删除此评价".into()));
    }

    sqlx::query("DELETE FROM course_reviews WHERE id = ?")
        .bind(id)
        .execute(&state.pool)
        .await?;

    Ok(Json(serde_json::json!({ "message": "评价已删除" })))
}

pub async fn like_review(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    auth: AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    let review = sqlx::query_as::<_, (i64, String)>(
        "SELECT user_id, status FROM course_reviews WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound("评价不存在".into()))?;

    if review.1 != "visible" {
        return Err(AppError::BadRequest("该评价不可见".into()));
    }

    let now = Utc::now().naive_utc();

    // Try insert like
    let result =
        sqlx::query("INSERT INTO review_likes (review_id, user_id, created_at) VALUES (?, ?, ?)")
            .bind(id)
            .bind(auth.id)
            .bind(&now)
            .execute(&state.pool)
            .await;

    match result {
        Ok(_) => {
            sqlx::query("UPDATE course_reviews SET like_count = like_count + 1 WHERE id = ?")
                .bind(id)
                .execute(&state.pool)
                .await?;

            // Award experience to review author (including self-likes)
            sqlx::query("UPDATE users SET experience = experience + 1 WHERE id = ?")
                .bind(review.0)
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

pub async fn unlike_review(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    auth: AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    let review = sqlx::query_as::<_, (i64,)>("SELECT user_id FROM course_reviews WHERE id = ?")
        .bind(id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(AppError::NotFound("评价不存在".into()))?;

    let deleted = sqlx::query("DELETE FROM review_likes WHERE review_id = ? AND user_id = ?")
        .bind(id)
        .bind(auth.id)
        .execute(&state.pool)
        .await?
        .rows_affected();

    if deleted > 0 {
        sqlx::query(
            "UPDATE course_reviews SET like_count = GREATEST(like_count - 1, 0) WHERE id = ?",
        )
        .bind(id)
        .execute(&state.pool)
        .await?;

        sqlx::query("UPDATE users SET experience = GREATEST(experience - 1, 0) WHERE id = ?")
            .bind(review.0)
            .execute(&state.pool)
            .await?;
    }

    Ok(Json(serde_json::json!({ "message": "已取消点赞" })))
}

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::error::AppError;
use crate::modules::AppState;

#[derive(Debug, Deserialize)]
pub struct CourseListQuery {
    pub keyword: Option<String>,
    pub teacher: Option<String>,
    pub category: Option<String>,
    pub sort: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct CourseListItem {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub teacher: String,
    pub credit: Option<f64>,
    pub category: String,
    pub description: String,
    pub rating_avg: Option<f64>,
    pub review_count: i64,
    pub material_count: i64,
}

#[derive(Debug, Serialize, FromRow)]
pub struct CourseDetail {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub teacher: String,
    pub credit: Option<f64>,
    pub category: String,
    pub semester: String,
    pub description: String,
    pub status: String,
    pub rating_avg: Option<f64>,
}

pub async fn list_courses(
    State(state): State<AppState>,
    Query(q): Query<CourseListQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(10).max(1).min(50);
    let offset = (page - 1) * page_size;
    let keyword = q.keyword.unwrap_or_default();
    let teacher = q.teacher.unwrap_or_default();
    let category = q.category.unwrap_or_default();

    let order = match q.sort.as_deref() {
        Some("rating_desc") => "rating_avg DESC",
        Some("rating_asc") => "rating_avg ASC",
        _ => "c.id DESC",
    };

    let courses = sqlx::query_as::<_, CourseListItem>(
        &format!(
            "SELECT c.id, c.code, c.name, COALESCE(c.teacher, '') as teacher, c.credit, COALESCE(c.category, '') as category, \
             COALESCE(c.description, '') as description, \
             CAST((SELECT AVG(r.rating) FROM course_reviews r WHERE r.course_id = c.id AND r.status = 'visible') AS DOUBLE) as rating_avg, \
             (SELECT COUNT(*) FROM course_reviews r WHERE r.course_id = c.id AND r.status = 'visible') as review_count, \
             (SELECT COUNT(*) FROM materials m WHERE m.course_id = c.id AND m.status = 'approved') as material_count \
             FROM courses c WHERE c.status = 'active' \
             AND (c.name LIKE ? OR c.code LIKE ? OR c.teacher LIKE ? OR ? = '') \
             AND (c.teacher LIKE ? OR ? = '') \
             AND (c.category LIKE ? OR ? = '') \
             ORDER BY {} LIMIT ? OFFSET ?",
            order
        )
    )
    .bind(format!("%{}%", keyword))
    .bind(format!("%{}%", keyword))
    .bind(format!("%{}%", keyword))
    .bind(&keyword)
    .bind(format!("%{}%", teacher))
    .bind(&teacher)
    .bind(format!("%{}%", category))
    .bind(&category)
    .bind(page_size)
    .bind(offset)
    .fetch_all(&state.pool)
    .await?;

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM courses c WHERE c.status = 'active' \
         AND (c.name LIKE ? OR c.code LIKE ? OR c.teacher LIKE ? OR ? = '') \
         AND (c.teacher LIKE ? OR ? = '') \
         AND (c.category LIKE ? OR ? = '')",
    )
    .bind(format!("%{}%", keyword))
    .bind(format!("%{}%", keyword))
    .bind(format!("%{}%", keyword))
    .bind(&keyword)
    .bind(format!("%{}%", teacher))
    .bind(&teacher)
    .bind(format!("%{}%", category))
    .bind(&category)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(serde_json::json!({
        "courses": courses,
        "total": total,
        "page": page,
        "page_size": page_size
    })))
}

pub async fn get_course(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<CourseDetail>, AppError> {
    let course = sqlx::query_as::<_, CourseDetail>(
        "SELECT id, code, name, COALESCE(teacher, '') as teacher, credit, COALESCE(category, '') as category, \
         COALESCE(semester, '') as semester, COALESCE(description, '') as description, status, \
         CAST((SELECT AVG(r.rating) FROM course_reviews r WHERE r.course_id = courses.id AND r.status = 'visible') AS DOUBLE) as rating_avg \
         FROM courses WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound("课程不存在".into()))?;

    Ok(Json(course))
}

mod config;
mod db;
mod error;
mod middleware;
mod modules;
mod redis;

use axum::{
    http::{header, HeaderValue, Method},
    routing::{delete, get, post, put},
    Router,
};
use modules::AppState;
use std::net::SocketAddr;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing_subscriber;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = config::Config::from_env();
    let pool = db::init_pool(&config.database_url).await;
    let redis = redis::init_redis(&config.redis_url).await;

    // Run migrations
    run_migrations(&pool)
        .await
        .expect("Database migrations failed");

    let state = AppState {
        pool,
        config: config.clone(),
        redis,
    };

    let cors_origins = config
        .cors_allowed_origins
        .iter()
        .map(|origin| origin.parse::<HeaderValue>().expect("Invalid CORS origin"))
        .collect::<Vec<_>>();
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(cors_origins))
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]);

    let public_router = Router::new()
        .route("/auth/send-code", post(modules::auth::send_code))
        .route("/auth/register", post(modules::auth::register))
        .route("/auth/login", post(modules::auth::login))
        .route("/courses", get(modules::courses::list_courses))
        .route("/courses/:id", get(modules::courses::get_course))
        .route(
            "/courses/:course_id/reviews",
            get(modules::reviews::list_reviews),
        )
        .route(
            "/courses/:course_id/materials",
            get(modules::materials::list_materials),
        )
        .route("/projects", get(modules::projects::list_projects))
        .route("/projects/:id", get(modules::projects::get_project));

    let user_router = Router::new()
        .route("/auth/me", get(modules::auth::get_me))
        .route(
            "/courses/:course_id/like-status",
            get(modules::reviews::get_course_like_status),
        )
        .route(
            "/courses/:course_id/reviews",
            post(modules::reviews::create_review),
        )
        .route("/reviews/:id", put(modules::reviews::update_review))
        .route("/reviews/:id", delete(modules::reviews::delete_review))
        .route("/reviews/:id/like", post(modules::reviews::like_review))
        .route("/reviews/:id/like", delete(modules::reviews::unlike_review))
        .route(
            "/courses/:course_id/materials",
            post(modules::materials::upload_material),
        )
        .route(
            "/materials/:id/download",
            get(modules::materials::download_material),
        )
        .route(
            "/materials/:id/like",
            post(modules::materials::like_material),
        )
        .route(
            "/materials/:id/like",
            delete(modules::materials::unlike_material),
        )
        .route(
            "/materials/:id",
            delete(modules::materials::delete_material),
        )
        .route("/projects", post(modules::projects::create_project))
        .route("/projects/:id", put(modules::projects::update_project))
        .route("/projects/:id", delete(modules::projects::delete_project))
        .route(
            "/projects/:id/status",
            put(modules::projects::update_project_status),
        )
        .route(
            "/projects/:id/apply",
            post(modules::projects::apply_project),
        )
        .route("/profile", get(modules::users::get_profile))
        .route("/profile", put(modules::users::update_profile))
        .route("/profile/password", put(modules::users::change_password))
        .route("/profile/materials", get(modules::users::get_my_materials))
        .route("/profile/reviews", get(modules::users::get_my_reviews))
        .route("/profile/projects", get(modules::users::get_my_projects))
        .route(
            "/profile/applications",
            get(modules::users::get_my_applications),
        )
        .route(
            "/profile/received-applications",
            get(modules::projects::list_received_applications),
        )
        .route(
            "/project-applications/:id/status",
            put(modules::projects::handle_application),
        )
        .route(
            "/profile/project-applications/:id",
            delete(modules::projects::delete_application),
        );

    let admin_router = Router::new()
        .route("/admin/dashboard", get(modules::admin::get_dashboard))
        .route("/admin/courses", post(modules::admin::create_course))
        .route("/admin/courses/:id", put(modules::admin::update_course))
        .route("/admin/courses/:id", delete(modules::admin::delete_course))
        .route("/admin/audit-items", get(modules::admin::list_audit_items))
        .route(
            "/admin/materials/:id/status",
            put(modules::admin::audit_material),
        )
        .route(
            "/admin/reviews/:id/status",
            put(modules::admin::update_review_status),
        )
        .route("/admin/users", get(modules::admin::list_users))
        .route(
            "/admin/users/:id/status",
            put(modules::admin::update_user_status),
        )
        .route("/admin/projects", get(modules::admin::list_projects))
        .route(
            "/admin/projects/:id/status",
            put(modules::admin::update_project_status),
        )
        .route("/admin/logs", get(modules::admin::get_admin_logs));

    let app = Router::new()
        .nest("/api", public_router)
        .nest("/api", user_router)
        .nest("/api", admin_router)
        .layer(cors)
        .with_state(state);

    let addr: SocketAddr = format!("{}:{}", config.app_host, config.app_port)
        .parse()
        .expect("Invalid address");

    tracing::info!("Server running on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn run_migrations(pool: &sqlx::MySqlPool) -> Result<(), sqlx::Error> {
    let migration_sql = include_str!("../migrations/001_init.sql");
    let migration_sql = migration_sql
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.starts_with("--") && !trimmed.starts_with("/*") && !trimmed.starts_with("*/")
        })
        .collect::<Vec<_>>()
        .join("\n");

    for statement in migration_sql.split(';') {
        let trimmed = statement.trim();
        if !trimmed.is_empty() {
            sqlx::query(trimmed).execute(pool).await?;
        }
    }

    tracing::info!("Migrations completed");
    Ok(())
}

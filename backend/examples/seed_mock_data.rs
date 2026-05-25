use bcrypt::{hash, DEFAULT_COST};
use chrono::{Duration, Utc};
use dotenvy::dotenv;
use sqlx::mysql::MySqlPoolOptions;
use std::env;
use std::path::PathBuf;

const DATA_SIZE: i64 = 50;
const TEST_PASSWORD: &str = "MockTest123!";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL")?;
    let upload_dir = env::var("UPLOAD_DIR").unwrap_or_else(|_| "./uploads".into());
    let pool = MySqlPoolOptions::new()
        .max_connections(2)
        .connect(&database_url)
        .await?;
    let mut tx = pool.begin().await?;

    clear_previous_mock_rows(&mut tx).await?;

    let now = Utc::now().naive_utc();
    let password_hash = hash(TEST_PASSWORD, DEFAULT_COST)?;

    for n in 1..=DATA_SIZE {
        let email = format!("mock_user_{n:03}@mail2.sysu.edu.cn");
        let nickname = format!("模拟用户-{n:03}");
        sqlx::query(
            "INSERT INTO users (email, password_hash, nickname, role, experience, status, created_at, updated_at) \
             VALUES (?, ?, ?, 'user', ?, 'active', ?, ?) \
             ON DUPLICATE KEY UPDATE password_hash = VALUES(password_hash), nickname = VALUES(nickname), \
             experience = VALUES(experience), status = 'active', updated_at = VALUES(updated_at)"
        )
        .bind(email)
        .bind(&password_hash)
        .bind(nickname)
        .bind((n * 3) as i32)
        .bind(now)
        .bind(now)
        .execute(&mut *tx)
        .await?;

        let code = format!("MOCK{n:03}");
        let name = format!("模拟课程-{n:03}");
        let teacher = format!("测试教师-{}", ((n - 1) % 10) + 1);
        let category = if n % 2 == 0 {
            "专业必修"
        } else {
            "专业选修"
        };
        sqlx::query(
            "INSERT INTO courses (code, name, teacher, credit, category, semester, description, status, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, '2025-2026-2', ?, 'active', ?, ?) \
             ON DUPLICATE KEY UPDATE name = VALUES(name), teacher = VALUES(teacher), credit = VALUES(credit), \
             category = VALUES(category), description = VALUES(description), status = 'active', updated_at = VALUES(updated_at)"
        )
        .bind(code)
        .bind(name)
        .bind(teacher)
        .bind(((n % 4) + 1) as f64)
        .bind(category)
        .bind(format!("MOCK 分页与课程详情测试数据，第 {n} 门课程。"))
        .bind(now)
        .bind(now)
        .execute(&mut *tx)
        .await?;
    }

    let admin_id: i64 =
        sqlx::query_scalar("SELECT id FROM users WHERE email = 'admin@mail2.sysu.edu.cn'")
            .fetch_one(&mut *tx)
            .await?;
    let mock_user_1 = user_id(&mut tx, 1).await?;
    let mock_user_2 = user_id(&mut tx, 2).await?;
    let mock_user_50 = user_id(&mut tx, 50).await?;
    let mock_course_1 = course_id(&mut tx, 1).await?;

    for days_ago in 0..30 {
        let active_users = 10 + ((days_ago * 7) % 31);
        for n in 1..=active_users {
            sqlx::query(
                "INSERT INTO user_daily_activity (user_id, activity_date, last_active_at) \
                 VALUES (?, ?, ?) ON DUPLICATE KEY UPDATE last_active_at = VALUES(last_active_at)",
            )
            .bind(user_id(&mut tx, n).await?)
            .bind(now.date() - Duration::days(days_ago))
            .bind(now - Duration::days(days_ago))
            .execute(&mut *tx)
            .await?;
        }
    }

    for n in 1..=DATA_SIZE {
        let created_at = now - Duration::minutes(DATA_SIZE - n);

        let (review_course_id, review_user_id) = if n <= 25 {
            (mock_course_1, user_id(&mut tx, n + 25).await?)
        } else {
            (course_id(&mut tx, n - 24).await?, mock_user_1)
        };
        let review_result = sqlx::query(
            "INSERT INTO course_reviews (course_id, user_id, rating, difficulty, workload, content, like_count, status, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, 1, 'visible', ?, ?)"
        )
        .bind(review_course_id)
        .bind(review_user_id)
        .bind(((n - 1) % 5 + 1) as i32)
        .bind(if n % 2 == 0 { "适中" } else { "较难" })
        .bind(if n % 3 == 0 { "较多" } else { "适中" })
        .bind(format!("MOCK评价-{n:03}：用于验证课程评价列表与个人中心分页。"))
        .bind(created_at)
        .bind(created_at)
        .execute(&mut *tx)
        .await?;
        let review_id = review_result.last_insert_id() as i64;

        sqlx::query("INSERT INTO review_likes (review_id, user_id, created_at) VALUES (?, ?, ?)")
            .bind(review_id)
            .bind(mock_user_2)
            .bind(created_at)
            .execute(&mut *tx)
            .await?;

        let material_status = "approved";
        let is_link = n % 10 == 0;
        let (original_name, file_path, file_size, file_type) = if is_link {
            (
                String::from("外部链接"),
                String::from("https://example.com/mock-material"),
                0_i64,
                String::from("text/uri-list"),
            )
        } else {
            (
                format!("mock-material-{n:03}.pdf"),
                format!("mock/mock-material-{n:03}.pdf"),
                147_i64,
                String::from("application/pdf"),
            )
        };
        let material_result = sqlx::query(
            "INSERT INTO materials (course_id, user_id, title, description, tag, original_name, file_path, file_size, file_type, download_count, like_count, status, created_at, updated_at) \
             VALUES (?, ?, ?, ?, '测试资料', ?, ?, ?, ?, ?, 1, ?, ?, ?)"
        )
        .bind(mock_course_1)
        .bind(mock_user_1)
        .bind(format!("MOCK资料-{n:03}"))
        .bind(format!("用于验证资料列表、下载与内容管理分页的模拟资料 {n:03}。"))
        .bind(original_name)
        .bind(file_path)
        .bind(file_size)
        .bind(file_type)
        .bind((n % 8) as i32)
        .bind(material_status)
        .bind(created_at)
        .bind(created_at)
        .execute(&mut *tx)
        .await?;
        let material_id = material_result.last_insert_id() as i64;

        sqlx::query(
            "INSERT INTO material_likes (material_id, user_id, created_at) VALUES (?, ?, ?)",
        )
        .bind(material_id)
        .bind(mock_user_2)
        .bind(created_at)
        .execute(&mut *tx)
        .await?;

        let owner_number = ((n - 1) / 5) + 1;
        let project_owner = user_id(&mut tx, owner_number).await?;
        let required_members = ((n % 4) + 2) as i32;
        let application_accepted = n % 4 == 0;
        let current_members = if application_accepted { 2 } else { 1 };
        let project_status = if n % 5 == 0 {
            "closed"
        } else if current_members >= required_members {
            "full"
        } else {
            "recruiting"
        };
        let project_result = sqlx::query(
            "INSERT INTO projects (user_id, title, type, tech_stack, description, requirements, required_members, current_members, contact, deadline, status, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(project_owner)
        .bind(format!("MOCK项目-{n:03}"))
        .bind(if n % 2 == 0 { "competition" } else { "course_project" })
        .bind(if n % 2 == 0 { "Vue, Rust, MySQL" } else { "Python, 数据分析" })
        .bind(format!("MOCK 招募描述 {n:03}，用于分页和项目详情测试。"))
        .bind("熟悉协作开发，能够按时交流进度。")
        .bind(required_members)
        .bind(current_members)
        .bind(format!("mock_user_{owner_number:03}@mail2.sysu.edu.cn"))
        .bind(now.date() + Duration::days(n + 30))
        .bind(project_status)
        .bind(created_at)
        .bind(created_at)
        .execute(&mut *tx)
        .await?;
        let project_id = project_result.last_insert_id() as i64;
        let applicant = mock_user_50;

        sqlx::query(
            "INSERT INTO project_applications (project_id, user_id, introduction, contact, status, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(project_id)
        .bind(applicant)
        .bind(format!("MOCK申请-{n:03}：希望参与该项目并负责功能开发。"))
        .bind(format!("mock_applicant_{n:03}@mail2.sysu.edu.cn"))
        .bind(if application_accepted { "accepted" } else { "pending" })
        .bind(created_at)
        .bind(created_at)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            "INSERT INTO admin_logs (admin_id, action, target_type, target_id, detail, created_at) \
             VALUES (?, 'seed_mock_data', 'project', ?, ?, ?)"
        )
        .bind(admin_id)
        .bind(project_id)
        .bind(format!("MOCK种子数据-{n:03}：生成测试项目"))
        .bind(created_at)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    write_mock_download_files(&upload_dir).await?;

    let summary = [
        ("users", "SELECT COUNT(*) FROM users WHERE nickname LIKE '模拟用户-%'"),
        ("courses", "SELECT COUNT(*) FROM courses WHERE code LIKE 'MOCK%'"),
        ("course_reviews", "SELECT COUNT(*) FROM course_reviews WHERE content LIKE 'MOCK评价-%'"),
        ("materials", "SELECT COUNT(*) FROM materials WHERE title LIKE 'MOCK资料-%'"),
        ("material_likes", "SELECT COUNT(*) FROM material_likes ml JOIN materials m ON ml.material_id = m.id WHERE m.title LIKE 'MOCK资料-%'"),
        ("review_likes", "SELECT COUNT(*) FROM review_likes rl JOIN course_reviews r ON rl.review_id = r.id WHERE r.content LIKE 'MOCK评价-%'"),
        ("projects", "SELECT COUNT(*) FROM projects WHERE title LIKE 'MOCK项目-%'"),
        ("project_applications", "SELECT COUNT(*) FROM project_applications pa JOIN projects p ON pa.project_id = p.id WHERE p.title LIKE 'MOCK项目-%'"),
        ("admin_logs", "SELECT COUNT(*) FROM admin_logs WHERE detail LIKE 'MOCK种子数据-%'"),
        ("user_daily_activity", "SELECT COUNT(*) FROM user_daily_activity a JOIN users u ON a.user_id = u.id WHERE u.email LIKE 'mock_user_%@mail2.sysu.edu.cn'"),
    ];

    println!("模拟数据准备完成，测试用户: mock_user_001@mail2.sysu.edu.cn / {TEST_PASSWORD}");
    for (table, query) in summary {
        let count: i64 = sqlx::query_scalar(query).fetch_one(&pool).await?;
        println!("{table}: {count}");
    }
    Ok(())
}

async fn clear_previous_mock_rows(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
) -> Result<(), sqlx::Error> {
    let queries = [
        "DELETE ml FROM material_likes ml JOIN materials m ON ml.material_id = m.id WHERE m.title LIKE 'MOCK资料-%' OR m.title LIKE 'MOCK-LINK-%' OR m.title LIKE 'MOCK-INSTANT-%'",
        "DELETE rl FROM review_likes rl JOIN course_reviews r ON rl.review_id = r.id JOIN courses c ON r.course_id = c.id WHERE c.code LIKE 'MOCK%'",
        "DELETE pa FROM project_applications pa JOIN projects p ON pa.project_id = p.id WHERE p.title LIKE 'MOCK项目-%' OR p.title LIKE 'MOCK流程验证-%' OR p.title LIKE 'MOCK-LIMIT-%' OR p.title LIKE 'MOCK-STATUS-%'",
        "DELETE al FROM admin_logs al JOIN materials m ON al.target_type = 'material' AND al.target_id = m.id WHERE m.title LIKE 'MOCK-LINK-%' OR m.title LIKE 'MOCK-INSTANT-%'",
        "DELETE FROM materials WHERE title LIKE 'MOCK资料-%' OR title LIKE 'MOCK-LINK-%' OR title LIKE 'MOCK-INSTANT-%'",
        "DELETE r FROM course_reviews r JOIN courses c ON r.course_id = c.id WHERE c.code LIKE 'MOCK%'",
        "DELETE FROM projects WHERE title LIKE 'MOCK项目-%' OR title LIKE 'MOCK流程验证-%' OR title LIKE 'MOCK-LIMIT-%' OR title LIKE 'MOCK-STATUS-%'",
        "DELETE FROM admin_logs WHERE detail LIKE 'MOCK种子数据-%'",
        "DELETE a FROM user_daily_activity a JOIN users u ON a.user_id = u.id WHERE u.email LIKE 'mock_user_%@mail2.sysu.edu.cn'",
    ];
    for query in queries {
        sqlx::query(query).execute(&mut **tx).await?;
    }
    Ok(())
}

async fn user_id(tx: &mut sqlx::Transaction<'_, sqlx::MySql>, n: i64) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT id FROM users WHERE email = ?")
        .bind(format!("mock_user_{n:03}@mail2.sysu.edu.cn"))
        .fetch_one(&mut **tx)
        .await
}

async fn course_id(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    n: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT id FROM courses WHERE code = ?")
        .bind(format!("MOCK{n:03}"))
        .fetch_one(&mut **tx)
        .await
}

async fn write_mock_download_files(upload_dir: &str) -> Result<(), std::io::Error> {
    let directory = PathBuf::from(upload_dir).join("mock");
    tokio::fs::create_dir_all(&directory).await?;
    let content = b"%PDF-1.1\n1 0 obj<</Type/Catalog>>endobj\ntrailer<</Root 1 0 R>>\n%%EOF\n";
    for n in 1..=DATA_SIZE {
        if n % 10 != 0 {
            tokio::fs::write(directory.join(format!("mock-material-{n:03}.pdf")), content).await?;
        }
    }
    Ok(())
}

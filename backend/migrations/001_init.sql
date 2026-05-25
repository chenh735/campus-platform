SET NAMES utf8mb4 COLLATE utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS users (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    email VARCHAR(128) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    nickname VARCHAR(64) NOT NULL,
    role VARCHAR(20) NOT NULL DEFAULT 'user',
    experience INT NOT NULL DEFAULT 0,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL
);

CREATE TABLE IF NOT EXISTS user_daily_activity (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    user_id BIGINT NOT NULL,
    activity_date DATE NOT NULL,
    last_active_at DATETIME NOT NULL,
    UNIQUE KEY unique_user_activity_date (user_id, activity_date),
    INDEX idx_activity_date (activity_date),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS courses (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    code VARCHAR(64) NOT NULL UNIQUE,
    name VARCHAR(128) NOT NULL,
    teacher VARCHAR(128) DEFAULT '',
    credit DOUBLE DEFAULT NULL,
    category VARCHAR(64) DEFAULT '',
    semester VARCHAR(64) DEFAULT '',
    description TEXT,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL
);

CREATE TABLE IF NOT EXISTS course_reviews (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    course_id BIGINT NOT NULL,
    user_id BIGINT NOT NULL,
    rating TINYINT NOT NULL,
    difficulty VARCHAR(20) DEFAULT NULL,
    workload VARCHAR(20) DEFAULT NULL,
    content TEXT NOT NULL,
    like_count INT NOT NULL DEFAULT 0,
    status VARCHAR(20) NOT NULL DEFAULT 'visible',
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL,
    UNIQUE KEY unique_course_user (course_id, user_id),
    FOREIGN KEY (course_id) REFERENCES courses(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS materials (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    course_id BIGINT NOT NULL,
    user_id BIGINT NOT NULL,
    title VARCHAR(128) NOT NULL,
    description TEXT,
    tag VARCHAR(64) DEFAULT '',
    original_name VARCHAR(255) NOT NULL,
    file_path VARCHAR(512) NOT NULL,
    file_size BIGINT NOT NULL,
    file_type VARCHAR(64) DEFAULT '',
    download_count INT NOT NULL DEFAULT 0,
    like_count INT NOT NULL DEFAULT 0,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL,
    FOREIGN KEY (course_id) REFERENCES courses(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS material_likes (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    material_id BIGINT NOT NULL,
    user_id BIGINT NOT NULL,
    created_at DATETIME NOT NULL,
    UNIQUE KEY unique_material_user (material_id, user_id),
    FOREIGN KEY (material_id) REFERENCES materials(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS review_likes (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    review_id BIGINT NOT NULL,
    user_id BIGINT NOT NULL,
    created_at DATETIME NOT NULL,
    UNIQUE KEY unique_review_user (review_id, user_id),
    FOREIGN KEY (review_id) REFERENCES course_reviews(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS projects (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    user_id BIGINT NOT NULL,
    title VARCHAR(128) NOT NULL,
    type VARCHAR(32) NOT NULL,
    tech_stack VARCHAR(255) DEFAULT '',
    description TEXT NOT NULL,
    requirements TEXT,
    required_members INT NOT NULL,
    current_members INT NOT NULL DEFAULT 1,
    contact VARCHAR(255) DEFAULT '',
    deadline DATE DEFAULT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'recruiting',
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS project_applications (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    project_id BIGINT NOT NULL,
    user_id BIGINT NOT NULL,
    introduction TEXT,
    contact VARCHAR(255) DEFAULT '',
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL,
    UNIQUE KEY unique_project_user (project_id, user_id),
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS admin_logs (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    admin_id BIGINT NOT NULL,
    action VARCHAR(64) NOT NULL,
    target_type VARCHAR(64) NOT NULL,
    target_id BIGINT DEFAULT NULL,
    detail TEXT,
    created_at DATETIME NOT NULL
);

-- Insert default admin user (password: admin123, needs to be changed after first login)
INSERT IGNORE INTO users (email, password_hash, nickname, role, experience, status, created_at, updated_at)
VALUES ('admin@mail2.sysu.edu.cn', '$2b$12$DVZIi02zqw7SH/SCgag6wuh/MpqHTPm7j974vfNQ2cwPrV7gdHH.y', '管理员', 'admin', 0, 'active', NOW(), NOW());

-- Insert some sample courses
INSERT IGNORE INTO courses (code, name, teacher, credit, category, semester, description, created_at, updated_at) VALUES
('SE101', '软件工程', '张老师', 3.0, '专业必修', '2025-2026-2', '本课程介绍软件工程的基本概念、开发过程、需求分析、软件设计、测试和项目管理等内容。', NOW(), NOW()),
('DB101', '数据库系统', '李老师', 3.0, '专业必修', '2025-2026-2', '学习关系数据库理论、SQL语言、数据库设计、事务管理和查询优化。', NOW(), NOW()),
('OS101', '操作系统', '王老师', 4.0, '专业必修', '2025-2026-2', '涵盖进程管理、内存管理、文件系统和I/O管理等操作系统核心概念。', NOW(), NOW()),
('CN101', '计算机网络', '赵老师', 3.0, '专业必修', '2025-2026-2', '学习网络协议分层、TCP/IP协议族、路由算法和网络安全基础。', NOW(), NOW()),
('AI101', '人工智能导论', '刘老师', 2.0, '专业选修', '2025-2026-2', '介绍人工智能的基本概念、搜索算法、知识表示和机器学习入门。', NOW(), NOW());

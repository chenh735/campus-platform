# 待补充事项

## 一、需要你提供的配置信息

### 1. MySQL 数据库密码

文件：`backend/.env`

```
DATABASE_URL=mysql://root:你的密码@127.0.0.1:3306/campus_platform
```

将 `你的密码` 替换为本地 MySQL root 密码。

### 2. SMTP 邮箱配置（发送验证码）

文件：`backend/.env`

```env
SMTP_HOST=smtp.163.com
SMTP_PORT=465
SMTP_USERNAME=你的163邮箱@163.com
SMTP_PASSWORD=你的SMTP授权码
SMTP_FROM=你的163邮箱@163.com
```

`SMTP_PASSWORD` 不是 163 邮箱登录密码，而是在 163 邮箱网页版 **设置 → POP3/SMTP/IMAP → 开启 SMTP 服务** 后生成的授权码。

验证码已通过 SMTP 发送。部署前必须用真实邮箱测试发送和注册流程；发送失败时接口会返回失败，不会显示验证码。

### 3. JWT 密钥

文件：`backend/.env`

```
JWT_SECRET=change_this_secret_to_a_random_string
```

生产环境需替换为随机强密钥，可用 `openssl rand -base64 32` 生成。

---

## 二、代码层面待完善

### 1. SMTP 邮件发送已接入

**位置**：`backend/src/modules/auth.rs` — `send_code` 函数（约第 50-100 行）

**现状**：验证码通过 SMTP 邮件发送；接口不返回验证码，也不在日志中输出验证码。

**部署前验证**：配置 `SMTP_HOST`、`SMTP_PORT`、`SMTP_USERNAME`、`SMTP_PASSWORD`、`SMTP_FROM` 后，用真实邮箱完成发送验证码与注册流程。

### 2. 管理员内容审核筛选已完成

**位置**：`backend/src/modules/admin.rs` — `list_audit_items` 函数（约第 135-168 行）

**现状**：资料审核支持按状态与关键词（标题、作者、课程）查询，并支持识别文件和链接资料。

### 3. 课程搜索已完成

**位置**：`backend/src/modules/courses.rs` — `list_courses` 函数（约第 53 行）

**现状**：支持按课程名称、课程代码、教师关键词搜索，也支持单独传入教师筛选参数。

### 4. `.gitignore` 已存在

**位置**：项目根目录

**现状**：项目根目录已包含 `.gitignore`。部署或提交前仍应确认以下敏感及构建内容未被提交：

```
# Rust
backend/target/

# Node
frontend/node_modules/
frontend/dist/

# 环境配置（含敏感信息）
.env

# 上传文件
uploads/

# IDE
.idea/
*.iml
```

---

## 三、运维相关待准备

| 事项 | 说明 |
|------|------|
| MySQL 数据库 | 推荐使用 Docker Compose 启动：`docker compose up -d mysql`，数据持久化到 `mysql-data` 卷 |
| Redis | 推荐使用 Docker Compose 启动：`docker compose up -d redis`，AOF 持久化到 `redis-data` 卷 |
| Docker 环境 | 需安装 Docker Desktop 或 Docker Engine，项目根目录已提供 `docker-compose.yml` |
| 上传目录 | 后端启动时会自动创建 `uploads/` 目录，确保有写入权限 |
| 域名和服务器 | 生产部署需要准备服务器、域名、配置 Nginx 和 systemd |

---

## 四、二期可扩展功能

按优先级从高到低排列（来自 `docs/01-功能规划.md`）：

| 优先级 | 功能 |
|--------|------|
| 已完成 | 项目申请处理（发布者审核申请） |
| P2 | 站内通知（资料审核结果、项目申请状态） |
| P2 | 课程复杂筛选和统计看板增强 |
| P3 | 即时聊天 |
| P3 | 全文搜索（Elasticsearch） |
| P3 | MinIO 对象存储迁移 |
| P3 | 移动端深度优化 |

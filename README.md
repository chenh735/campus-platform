# 软件工程学院学习协作平台

一个面向学院内部课程学习、资料共享和项目招募的 Web 平台，提供课程评价、学习资料上传、项目/比赛招募、个人中心和管理后台等功能。

## 在线访问与测试账号

在线地址：

```text
https://campus.chenh735blog.top
```

默认管理员测试账号：

```text
邮箱：admin@mail2.sysu.edu.cn
密码：admin123
```

> 生产环境请及时修改默认管理员密码，并替换 `.env` 中的数据库密码、JWT 密钥和 SMTP 授权码。

## 平台定位

本平台主要解决学院课程学习和项目协作中的信息分散问题：

- 课程学习：沉淀课程评分、课程评价、难度和工作量反馈。
- 资料共享：集中管理课程资料，支持文件资料和外部链接资料。
- 项目招募：发布课程项目、比赛组队和协作招募，并处理加入申请。
- 后台管理：为管理员提供课程、用户、资料和项目的管理入口。

平台面向三类角色：

- 游客：查看课程、资料和公开项目招募。
- 普通用户：评价课程、上传资料、发布项目、申请加入项目。
- 管理员：维护课程、处理内容、管理用户和项目。

## 用户端功能

### 课程

- 查看课程列表和课程详情。
- 支持课程分页展示。
- 支持按课程名称、课程代码、教师搜索。
- 支持按课程类别筛选和按评分等条件排序。
- 查看课程评分、评价数量和资料数量。

### 课程评价

- 登录用户可以发布课程评价。
- 评价内容包含评分、难度、工作量和文字评价。
- 同一用户对同一课程只能评价一次。
- 支持评价分页展示。
- 支持点赞和取消点赞。

### 学习资料

- 登录用户可以上传课程学习资料。
- 支持文件资料，例如 PDF、DOCX、PPT、ZIP、图片等。
- 支持 `http/https` 外部链接资料。
- 上传后的资料立即展示。
- 管理员可以在后台隐藏或删除违规资料。
- 支持资料下载、点赞和取消点赞。

### 项目招募

- 用户可以发布课程项目或比赛组队招募。
- 招募信息支持类型、技术方向、人数、截止日期和描述。
- 用户可以申请加入他人发布的项目。
- 发布者可以在个人中心查看收到的申请，并选择同意或拒绝。
- 发布者可以关闭或隐藏自己的招募。
- 每个用户最多保留 5 个未关闭、未隐藏的招募。

### 个人中心

- 查看个人资料、等级和经验值。
- 管理我的资料、我的评价、我的招募、我的申请。
- 查看收到的项目申请。
- 修改昵称和密码。

## 管理端功能

- 后台首页：查看用户数、课程数、资料数、项目招募数和待处理数据。
- 活跃统计：查看今日活跃人数和每日活跃趋势。
- 课程管理：新增、编辑、下架课程。
- 内容管理：搜索资料，隐藏或删除资料。
- 用户管理：搜索用户，禁用或恢复用户。
- 项目管理：搜索项目，隐藏或关闭项目。
- 操作日志：记录后台关键操作。

## 规则与限制

- 注册邮箱限制为指定域名，默认允许 `mail2.sysu.edu.cn`。
- 文件上传单文件大小由 `MAX_UPLOAD_MB` 控制，默认 50 MB。
- 每个用户上传的文件资料总大小限制为 1 GiB。
- 链接资料不占用文件容量。
- 点赞会增加作者经验值，取消点赞会回退对应经验值。
- 管理员权限由后端校验，不只依赖前端页面控制。

## 技术栈

| 模块 | 技术 | 说明 |
| --- | --- | --- |
| 前端 | Vue 3 + Vite + TypeScript | 单页应用开发 |
| UI | Element Plus | 表格、表单、弹窗、分页等组件 |
| 状态管理 | Pinia | 管理用户信息和登录状态 |
| 路由 | Vue Router | 用户端和管理端页面路由 |
| 后端 | Rust + Axum + Tokio | 异步 REST API 服务 |
| 数据库 | MySQL 8 | 保存核心业务数据 |
| 数据访问 | SQLx | SQL 查询和数据映射 |
| 缓存 | Redis | 验证码和频率限制 |
| 鉴权 | JWT | 登录认证和接口鉴权 |
| 部署 | Docker Compose | 多服务容器编排 |
| HTTPS | Caddy | 自动证书和反向代理 |

## 系统运行结构

生产环境运行链路：

```text
浏览器
  -> Caddy，负责 HTTPS 和反向代理
  -> 前端 Nginx 容器，托管静态页面
  -> Rust Axum 后端，提供 /api 接口
  -> MySQL / Redis
```

核心容器：

- `campus-mysql`：MySQL 数据库。
- `campus-redis`：Redis 缓存。
- `campus-backend`：Rust 后端 API。
- `campus-frontend`：前端静态资源和 API 代理。
- `campus-caddy`：HTTPS 入口和反向代理。

## 本地开发

### 环境要求

- Node.js 18+
- Rust 1.85+
- Docker Desktop 或 Docker Engine + Docker Compose

### 启动 MySQL 和 Redis

```bash
docker compose up -d
```

### 配置后端

进入 `backend/` 目录，创建 `.env`：

```env
APP_HOST=127.0.0.1
APP_PORT=8080
DATABASE_URL=mysql://root:your_password@127.0.0.1:3306/campus_platform
REDIS_URL=redis://127.0.0.1:6379
JWT_SECRET=replace_with_a_long_random_secret
UPLOAD_DIR=./uploads
MAX_UPLOAD_MB=50
SMTP_HOST=smtp.example.com
SMTP_PORT=465
SMTP_USERNAME=your_email@example.com
SMTP_PASSWORD=your_smtp_auth_code
SMTP_FROM=your_email@example.com
CORS_ALLOWED_ORIGINS=http://localhost:3000
```

启动后端：

```bash
cd backend
cargo run --release
```

### 启动前端

```bash
cd frontend
npm install
npm run dev
```

默认访问：

```text
http://localhost:3000
```

## Docker 生产部署

### 准备环境变量

```bash
cp .env.docker.example .env
nano .env
```

生产环境需要修改：

```env
WEB_PORT=127.0.0.1:8081

MYSQL_DATABASE=campus_platform
MYSQL_USER=campus_user
MYSQL_PASSWORD=replace_with_strong_password
MYSQL_ROOT_PASSWORD=replace_with_another_strong_password

JWT_SECRET=replace_with_long_random_secret
MAX_UPLOAD_MB=50

SMTP_HOST=smtp.example.com
SMTP_PORT=465
SMTP_USERNAME=your_email@example.com
SMTP_PASSWORD=your_smtp_auth_code
SMTP_FROM=your_email@example.com

ALLOWED_EMAIL_DOMAINS=mail2.sysu.edu.cn
CORS_ALLOWED_ORIGINS=https://your-domain.example.com
```

生成随机 JWT 密钥：

```bash
openssl rand -hex 32
```

### 启动服务

```bash
docker compose -f docker-compose.prod.yml up -d --build
```

查看服务状态：

```bash
docker compose -f docker-compose.prod.yml ps
```

查看后端日志：

```bash
docker logs --tail 100 campus-backend
```

## HTTPS 配置

域名解析到服务器公网 IP 后，可以使用 Caddy 提供 HTTPS。

Caddyfile 示例：

```caddyfile
your-domain.example.com {
    encode zstd gzip
    reverse_proxy 127.0.0.1:8081
}
```

Caddy 容器可使用 host network 运行：

```bash
docker run -d \
  --name campus-caddy \
  --restart unless-stopped \
  --network host \
  -v /var/www/campus-platform/caddy/Caddyfile:/etc/caddy/Caddyfile \
  -v campus-caddy-data:/data \
  -v campus-caddy-config:/config \
  caddy:2
```

验证接口：

```bash
curl -i https://your-domain.example.com/api/courses?page_size=10
```

正常情况下应返回 `HTTP/2 200`。

## 常见测试功能

部署后建议测试以下功能：

- 注册验证码发送和邮箱域名限制。
- 普通用户登录、退出、修改资料。
- 课程列表分页、搜索、筛选、排序。
- 课程评价发布、点赞、取消点赞。
- 文件资料上传、链接资料上传、资料下载。
- 项目招募发布、申请加入、发布者处理申请。
- 用户最多保留 5 个有效招募的限制。
- 后台课程管理、内容管理、用户管理、项目管理。
- 今日活跃人数统计。
- HTTPS 访问和 `/api` 接口访问。

## 注意事项

- `.env`、`backend/.env`、真实密码、SMTP 授权码和 JWT 密钥不要提交到 Git 仓库。
- 生产环境应尽快修改默认管理员密码。
- 不要执行带 `-v` 的 `docker compose down`，除非明确要删除数据库和上传文件数据。
- 如果只修改前端，可以只重新构建前端服务：

```bash
docker compose -f docker-compose.prod.yml build --no-deps frontend
docker compose -f docker-compose.prod.yml up -d --no-deps frontend
```

- 如果接口返回 502，建议按顺序检查后端端口、前端代理和 Caddy 反向代理配置。

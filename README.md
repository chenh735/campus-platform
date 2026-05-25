# 校园课程学习与项目协作平台

软件工程学院内部课程学习与项目协作平台，支持课程评价、学习资料共享、项目/比赛组队招募。

## 技术栈

| 层级 | 技术 | 说明 |
|------|------|------|
| 前端 | Vue 3 + Vite + TypeScript | SPA 单页应用 |
| UI 组件库 | Element Plus | 表格、表单、弹窗、分页等 |
| 状态管理 | Pinia | 管理用户信息、Token、全局状态 |
| 路由 | Vue Router | 用户端和管理员端页面路由 |
| 后端 | Rust + Axum + Tokio | 高性能 REST API |
| 数据库 | MySQL 8 | 关系型数据存储 |
| 数据库访问 | SQLx | 编译期 SQL 检查 |
| 缓存 | Redis | 邮箱验证码、发送频率限制 |
| 鉴权 | JWT | 无状态认证 |

## 快速开始

### 前置依赖

- Node.js >= 18
- Rust >= 1.75
- Docker Desktop（或 Docker Engine + Docker Compose）

> MySQL 8 和 Redis 7 通过 Docker Compose 启动，无需手动安装。

### 1. 启动 MySQL 和 Redis（Docker Compose）

```bash
# 在项目根目录执行
docker compose up -d
```

MySQL 8 和 Redis 7 将通过 Docker 容器启动：
- MySQL 端口 `3306`，数据库 `campus_platform` 自动创建，迁移脚本自动执行
- Redis 端口 `6379`，AOF 持久化已开启
- 数据持久化到 Docker 命名卷（`mysql-data`、`redis-data`），容器删除后数据不丢失

### 2. 配置后端

进入 `backend/` 目录，编辑 `.env` 文件：

```env
APP_HOST=127.0.0.1
APP_PORT=8080
DATABASE_URL=mysql://root:你的密码@127.0.0.1:3306/campus_platform
REDIS_URL=redis://127.0.0.1:6379
JWT_SECRET=随机生成一个密钥字符串
UPLOAD_DIR=./uploads
MAX_UPLOAD_MB=50
SMTP_HOST=smtp.example.com
SMTP_PORT=465
SMTP_USERNAME=example@example.com
SMTP_PASSWORD=your_password
SMTP_FROM=example@example.com
CORS_ALLOWED_ORIGINS=http://localhost:3000
```

**重要**：SMTP 邮箱配置是发送验证码的关键依赖，详细获取方式见下文「配置邮箱验证码」章节。

### 3. 启动后端

```bash
cd backend
cargo run --release
```

后端启动时会自动执行数据库迁移（创建表和示例数据），包括：
- 默认管理员账号：`admin@mail2.sysu.edu.cn` / `admin123`
- 5 门示例课程（软件工程、数据库系统、操作系统、计算机网络、人工智能导论）

### 4. 配置并启动前端

```bash
cd frontend
npm install
npm run dev
```

前端默认运行在 `http://localhost:3000`，API 请求通过 Vite 代理转发到后端 `http://127.0.0.1:8080`。

### 5. 访问

- 用户端：`http://localhost:3000`
- 管理后台：用管理员账号登录后访问 `http://localhost:3000/admin`

---

## 配置邮箱验证码

平台注册需要发送邮箱验证码，使用 SMTP 协议。

### 获取 163 邮箱 SMTP 授权码（推荐）

1. 登录 163 邮箱网页版（https://mail.163.com）
2. 点击 **设置** → **POP3/SMTP/IMAP**
3. 找到 **SMTP 服务**，点击 **开启**
4. 按照提示设置授权码（可能需要短信验证）
5. 系统会生成一个授权码，将其填入 `.env` 文件的 `SMTP_PASSWORD` 字段

### 配置示例（使用 163 邮箱）

```env
SMTP_HOST=smtp.163.com
SMTP_PORT=465
SMTP_USERNAME=your_email@163.com
SMTP_PASSWORD=你的SMTP授权码
SMTP_FROM=your_email@163.com
```

### 使用其他邮箱

| 邮箱服务商 | SMTP_HOST | SMTP_PORT |
|-----------|-----------|-----------|
| QQ 邮箱 | smtp.qq.com | 465 |
| 163 邮箱 | smtp.163.com | 465 |
| Gmail | smtp.gmail.com | 587 |
| Outlook | smtp-mail.outlook.com | 587 |

> **注意**：Gmail 需要开启"应用专用密码"；部分邮箱需要先开启 SMTP 服务才能使用。

### 验证码发送说明

验证码仅通过配置的 SMTP 邮箱发送，不会返回给前端或输出到后端日志。正式部署前必须配置并实测 SMTP；未配置或发送失败时，接口会提示发送失败，用户可以在修复配置后重新发送。

---

## 编译

### 前端编译

```bash
cd frontend
npm run build
# 产出在 dist/ 目录
```

### 后端编译

```bash
cd backend
cargo build --release
# 产出在 target/release/campus-platform
```

**类型检查（不产出文件）：**

```bash
# 前端
cd frontend && npx vue-tsc --noEmit

# 后端
cd backend && cargo check
```

---

## 测试

当前 MVP 阶段，建议通过以下方式手动验证功能：

### 账号测试
- 非 `@mail2.sysu.edu.cn` 邮箱不能发送验证码
- 正确验证码可以注册，错误或过期验证码不能注册
- 禁用用户不能登录

### 课程测试
- 游客可以查看课程列表和详情
- 普通用户可以发表评价（同一课程只能评价一次）
- 编辑评价后课程评价列表更新
- 管理员可以隐藏违规评价

### 资料测试
- 普通用户可以上传允许类型的文件（PDF/DOCX/PPT/ZIP/图片）
- 超过 50MB 的文件不能上传
- 用户可以提交 `http/https` 链接资料，非法链接应被拒绝
- 同一用户文件资料累计达到 1 GiB 后不能继续上传文件
- 普通用户上传文件/链接资料后立即可见，管理员可后续隐藏或删除
- 点赞资料后作者经验值 +1，取消点赞 -1

### 项目招募测试
- 普通用户可以发布招募
- 每位用户最多同时保有 5 个未关闭、未隐藏的招募
- 发布者可以关闭或隐藏自己的招募，且隐藏后不在用户端公开列表显示
- 登录用户可以申请加入（不能重复申请同一项目）
- 发布者可以在个人中心查看并处理申请

### 权限测试
- 未登录用户访问个人中心会跳转登录页
- 普通用户访问 `/admin` 会被拒绝
- 用户不能编辑他人发布的评价、资料或项目

---

## 功能概要

### 用户端
- 邮箱验证码注册/登录（限 `@mail2.sysu.edu.cn`）
- 课程列表与详情（搜索、筛选、排序）
- 课程评分与评价（1-5 分，难度/工作量标签）
- 学习资料上传与下载（PDF/DOCX/PPT/ZIP/图片或外部链接，上传后立即可见）
- 资料点赞 / 评价点赞
- 经验值与等级系统（Lv1-Lv5）
- 项目/比赛招募（发布、查看、申请加入、处理申请、关闭/隐藏）
- 个人中心（资料、评价、招募、申请管理）

### 管理员端
- 系统统计看板（今日活跃人数、近 30 天每日活跃趋势）
- 课程管理（增删改）
- 内容管理（学习资料搜索、隐藏、删除）
- 用户管理（查看、禁用/恢复）
- 项目管理（隐藏/关闭违规招募）
- 操作日志

### 经验值等级规则

| 等级 | 经验值范围 |
|------|-----------|
| Lv1 | 0 - 9 |
| Lv2 | 10 - 29 |
| Lv3 | 30 - 59 |
| Lv4 | 60 - 99 |
| Lv5 | 100 及以上 |

| 行为 | 经验值变化 |
|------|-----------|
| 他人点赞自己的学习资料 | +1 |
| 他人取消点赞自己的学习资料 | -1 |
| 他人点赞自己的课程评价 | +1 |
| 他人取消点赞自己的课程评价 | -1 |

### 预设账号

| 类型 | 邮箱 | 密码 |
|------|------|------|
| 管理员 | `admin@mail2.sysu.edu.cn` | `admin123` |

---

## 部署

推荐使用 Docker 部署整站。生产版 Compose 会同时启动：

- `campus-mysql`：MySQL 8
- `campus-redis`：Redis 7
- `campus-backend`：Rust 后端 API
- `campus-frontend`：Nginx 托管前端静态文件并反向代理 `/api`

详细部署说明见：[docs/docker-deploy.md](docs/docker-deploy.md)。

### Docker 生产部署

服务器推荐使用 Ubuntu 22.04/24.04。先安装 Docker：

```bash
sudo apt update
sudo apt install -y ca-certificates curl git
curl -fsSL https://get.docker.com | sudo sh
sudo usermod -aG docker $USER
```

重新登录服务器后，进入项目目录并创建生产配置：

```bash
cp .env.docker.example .env
nano .env
```

`.env` 中必须修改：

```env
WEB_PORT=80

MYSQL_DATABASE=campus_platform
MYSQL_USER=campus_user
MYSQL_PASSWORD=换成强密码
MYSQL_ROOT_PASSWORD=换成另一个强密码

JWT_SECRET=换成随机长密钥
MAX_UPLOAD_MB=50

SMTP_HOST=smtp.example.com
SMTP_PORT=465
SMTP_USERNAME=你的邮箱
SMTP_PASSWORD=邮箱SMTP授权码
SMTP_FROM=你的邮箱

ALLOWED_EMAIL_DOMAINS=mail2.sysu.edu.cn
CORS_ALLOWED_ORIGINS=http://你的服务器IP
```

生成 `JWT_SECRET`：

```bash
openssl rand -hex 32
```

启动整站：

```bash
docker compose -f docker-compose.prod.yml up -d --build
```

查看运行状态：

```bash
docker compose -f docker-compose.prod.yml ps
```

如果看到 `mysql`、`redis`、`backend`、`frontend` 都是运行状态，就说明整站已启动。浏览器访问：

```text
http://你的服务器IP
```

查看后端日志：

```bash
docker logs -f campus-backend
```

> 注意：不要执行 `docker compose -f docker-compose.prod.yml down -v`，除非你明确想删除数据库和上传文件数据。`-v` 会删除 Docker volume。

### Docker 数据持久化

生产版 Compose 使用 Docker volume 保存数据：

- `mysql-data`：数据库
- `redis-data`：Redis 数据
- `uploads`：用户上传资料

执行下面命令不会清空数据：

```bash
docker compose -f docker-compose.prod.yml down
docker compose -f docker-compose.prod.yml up -d
```

执行下面命令会清空数据：

```bash
docker compose -f docker-compose.prod.yml down -v
```

### 可选：传统生产环境构建

**前端构建**：

```bash
cd frontend
npm run build
# 产出在 dist/ 目录
```

**后端构建**：

```bash
cd backend
cargo build --release
# 产出在 target/release/campus-platform
```

### 服务器目录结构

```
/var/www/campus-platform/
  frontend/dist/     # 前端静态文件
  backend/           # Rust 二进制 + .env
  uploads/materials/ # 学习资料文件
```

### Nginx 配置

```nginx
server {
    listen 80;
    server_name example.com;

    root /var/www/campus-platform/frontend/dist;
    index index.html;

    location / {
        try_files $uri $uri/ /index.html;
    }

    location /api/ {
        proxy_pass http://127.0.0.1:8080/api/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    }
}
```

### 后端环境变量（生产环境）

```env
APP_HOST=127.0.0.1
APP_PORT=8080
DATABASE_URL=mysql://user:password@127.0.0.1:3306/campus_platform
REDIS_URL=redis://127.0.0.1:6379
JWT_SECRET=生产环境请使用随机生成的强密钥
UPLOAD_DIR=/var/www/campus-platform/uploads
MAX_UPLOAD_MB=50
SMTP_HOST=smtp.qq.com
SMTP_PORT=465
SMTP_USERNAME=your_email@qq.com
SMTP_PASSWORD=your_smtp_auth_code
SMTP_FROM=your_email@qq.com
CORS_ALLOWED_ORIGINS=https://your-domain.example.com
```

### 传统部署初始化流程

1. 启动 Docker 中间件：`docker compose up -d`（MySQL + Redis，数据自动持久化）
2. 配置 `backend/.env` 文件
3. 启动后端（自动执行数据库迁移并创建初始管理员和示例课程）
4. 创建 `uploads` 目录并确保后端有写入权限
5. 构建前端并复制 `dist/` 到 Nginx 静态目录
6. 构建 Rust 后端二进制文件
7. 使用 systemd 或直接运行后端
8. 配置 Nginx 并重启

---

## 上传到 GitHub

新建 GitHub 仓库后，推荐把本项目目录作为独立 Git 仓库上传。不要在桌面根目录直接提交，否则可能把桌面上的其他文件一起提交。

进入项目目录：

```bash
cd C:\Users\chenhao\Desktop\SoftEngShare
```

如果当前目录还不是独立 Git 仓库，先初始化：

```bash
git init
git branch -M main
```

检查将要提交的文件：

```bash
git status --short
```

确认 `.env`、`backend/.env`、`frontend/node_modules/`、`frontend/dist/`、`backend/target/`、上传文件目录没有进入提交列表后，提交代码：

```bash
git add .
git commit -m "Initial commit"
```

绑定你刚创建的远程仓库：

```bash
git remote add origin https://github.com/chenh735/campus-platform.git
```

推送到 GitHub：

```bash
git push -u origin main
```

如果提示 `origin already exists`，说明已经绑定过远程仓库，可以改用：

```bash
git remote set-url origin https://github.com/chenh735/campus-platform.git
git push -u origin main
```

如果 GitHub 要求登录，按提示在浏览器登录，或使用 Personal Access Token 作为 HTTPS 密码。

---

## 项目结构

```
backend/
  src/
    main.rs               # 入口，路由配置、数据库迁移
    config.rs             # 环境变量配置
    db.rs                 # MySQL 连接池
    error.rs              # 统一错误处理和响应
    redis.rs              # Redis 连接初始化
    middleware/
      auth.rs             # JWT 鉴权提取器（FromRequestParts）
    modules/
      mod.rs              # 模块声明、AppState 定义
      auth.rs             # 注册、登录、验证码发送
      users.rs            # 个人中心、经验值计算
      courses.rs          # 课程列表与详情查询
      reviews.rs          # 课程评价与点赞
      materials.rs        # 学习资料上传、下载、点赞
      projects.rs         # 项目招募发布、申请
      admin.rs            # 后台管理（仪表盘、内容管理、用户管理）
      upload.rs           # 文件类型校验工具
  migrations/
    001_init.sql          # 数据库建表与初始数据

frontend/
  src/
    api/                  # API 接口封装（axios）
    layouts/              # 页面布局组件（DefaultLayout, AdminLayout）
    router/               # 路由配置（含权限守卫）
    stores/               # Pinia 状态管理（auth store）
    types/                # TypeScript 类型定义
    views/
      auth/               # 登录页、注册页
      courses/            # 课程列表、课程详情
      projects/           # 项目列表、发布、详情
      profile/            # 个人中心
      admin/              # 管理后台（Dashboard, 课程管理, 内容管理, 用户管理, 项目管理）
```

---

## API 接口概览

### 鉴权
| 方法 | 路径 | 说明 | 权限 |
|------|------|------|------|
| POST | `/api/auth/send-code` | 发送邮箱验证码 | 游客 |
| POST | `/api/auth/register` | 注册 | 游客 |
| POST | `/api/auth/login` | 登录 | 游客 |
| GET | `/api/auth/me` | 获取当前用户信息 | 登录用户 |

### 课程
| 方法 | 路径 | 说明 | 权限 |
|------|------|------|------|
| GET | `/api/courses` | 课程列表（支持搜索/筛选/排序/分页） | 游客 |
| GET | `/api/courses/:id` | 课程详情 | 游客 |
| POST | `/api/admin/courses` | 新增课程 | 管理员 |
| PUT | `/api/admin/courses/:id` | 编辑课程 | 管理员 |
| DELETE | `/api/admin/courses/:id` | 下架课程 | 管理员 |

### 评价
| 方法 | 路径 | 说明 | 权限 |
|------|------|------|------|
| GET | `/api/courses/:course_id/reviews` | 获取课程评价 | 游客 |
| POST | `/api/courses/:course_id/reviews` | 新增评价 | 登录用户 |
| PUT | `/api/reviews/:id` | 编辑评价 | 作者 |
| DELETE | `/api/reviews/:id` | 删除评价 | 作者 |
| POST | `/api/reviews/:id/like` | 点赞 | 登录用户 |
| DELETE | `/api/reviews/:id/like` | 取消点赞 | 登录用户 |

### 学习资料
| 方法 | 路径 | 说明 | 权限 |
|------|------|------|------|
| GET | `/api/courses/:course_id/materials` | 获取课程资料 | 游客 |
| POST | `/api/courses/:course_id/materials` | 上传资料 | 登录用户 |
| GET | `/api/materials/:id/download` | 下载资料 | 登录用户 |
| POST | `/api/materials/:id/like` | 点赞 | 登录用户 |
| DELETE | `/api/materials/:id/like` | 取消点赞 | 登录用户 |

### 项目招募
| 方法 | 路径 | 说明 | 权限 |
|------|------|------|------|
| GET | `/api/projects` | 项目列表 | 游客 |
| GET | `/api/projects/:id` | 项目详情 | 游客 |
| POST | `/api/projects` | 发布项目 | 登录用户 |
| PUT | `/api/projects/:id` | 编辑项目 | 作者 |
| DELETE | `/api/projects/:id` | 删除项目 | 作者 |
| POST | `/api/projects/:id/apply` | 申请加入 | 登录用户 |

### 个人中心
| 方法 | 路径 | 说明 | 权限 |
|------|------|------|------|
| GET | `/api/profile` | 获取个人信息 | 登录用户 |
| PUT | `/api/profile` | 修改昵称 | 登录用户 |
| PUT | `/api/profile/password` | 修改密码 | 登录用户 |
| GET | `/api/profile/project-applications` | 我的申请记录 | 登录用户 |

### 管理后台
| 方法 | 路径 | 说明 | 权限 |
|------|------|------|------|
| GET | `/api/admin/dashboard` | 系统统计 | 管理员 |
| GET | `/api/admin/users` | 用户列表 | 管理员 |
| PUT | `/api/admin/users/:id/status` | 禁用/恢复用户 | 管理员 |
| GET | `/api/admin/audit-items` | 学习资料内容列表 | 管理员 |
| PUT | `/api/admin/materials/:id/status` | 修改资料状态 | 管理员 |
| DELETE | `/api/materials/:id` | 删除学习资料 | 资料作者/管理员 |
| PUT | `/api/admin/reviews/:id/status` | 修改评价状态 | 管理员 |
| PUT | `/api/admin/projects/:id/status` | 修改项目状态 | 管理员 |
| GET | `/api/admin/logs` | 操作日志 | 管理员 |

---

## 数据库表结构

| 表名 | 说明 |
|------|------|
| `users` | 用户（含管理员） |
| `courses` | 课程 |
| `course_reviews` | 课程评价 |
| `materials` | 学习资料 |
| `material_likes` | 资料点赞记录 |
| `review_likes` | 评价点赞记录 |
| `projects` | 项目招募 |
| `project_applications` | 项目申请记录 |
| `admin_logs` | 管理员操作日志 |

详细字段定义见 `backend/migrations/001_init.sql`。

---

## 设计文档

详细的设计文档在 `docs/` 目录下：

- [01-功能规划](docs/01-功能规划.md) — 功能范围、用户角色、MVP 定义
- [02-原型设计](docs/02-原型设计.md) — 页面原型和交互流程
- [03-技术选型](docs/03-技术选型.md) — 技术栈选择和理由
- [04-系统设计](docs/04-系统设计.md) — 系统架构、接口设计、数据库设计

---

## 开发说明

### 权限控制

- 所有接口统一以 `/api` 开头
- 登录后请求头携带 `Authorization: Bearer <token>`
- 前端在 axios 拦截器中统一处理 401（跳转登录页）和 403（提示无权限）
- 管理员接口每次从数据库确认用户角色和状态，避免角色变更后 Token 长期有效

### 数据库迁移

后端启动时自动执行 `migrations/001_init.sql`，所有建表语句使用 `CREATE TABLE IF NOT EXISTS`，初始数据使用 `INSERT IGNORE INTO`，可安全重复执行。

### 内容管理

- 学习资料：默认 `approved`（已发布），普通用户上传后立即可下载/访问，管理员可隐藏或删除违规资料
- 课程评价：默认 `visible`（可见），管理员可隐藏违规评价
- 项目招募：默认 `recruiting`（招募中），管理员可隐藏违规项目

### 文件上传

- 默认支持类型：PDF、DOCX、PPT/PPTX、ZIP、JPG/JPEG、PNG、GIF
- 支持提交 `http/https` 外部链接资料，链接不占用文件容量
- 默认大小限制：50MB（可通过 `MAX_UPLOAD_MB` 配置）
- 每位用户文件资料累计大小限制：1 GiB
- 文件存储路径：`uploads/materials/{year}/{month}/{uuid}_{original_name}`
- 下载时需登录且资料状态为 `approved`

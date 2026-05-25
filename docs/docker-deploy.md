# Docker 生产部署指南

本文档适用于一台 Linux 服务器上部署本项目，推荐系统为 Ubuntu 22.04/24.04。

## 1. 服务器准备

安装 Docker 和 Compose 插件：

```bash
sudo apt update
sudo apt install -y ca-certificates curl git
curl -fsSL https://get.docker.com | sudo sh
sudo usermod -aG docker $USER
```

重新登录服务器后确认版本：

```bash
docker --version
docker compose version
```

如果云服务器有安全组/防火墙，至少开放：

- `80/tcp`：HTTP 访问
- `443/tcp`：HTTPS，配置域名证书后使用

## 2. 上传项目

推荐放在：

```bash
sudo mkdir -p /var/www
sudo chown -R $USER:$USER /var/www
cd /var/www
git clone <你的仓库地址> campus-platform
cd campus-platform
```

如果没有 Git 仓库，也可以把整个项目目录上传到 `/var/www/campus-platform`。

## 3. 配置环境变量

复制模板：

```bash
cp .env.docker.example .env
```

编辑 `.env`：

```bash
nano .env
```

必须修改这些值：

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

如果你有域名，`CORS_ALLOWED_ORIGINS` 改成：

```env
CORS_ALLOWED_ORIGINS=https://你的域名
```

注意：`MYSQL_PASSWORD` 尽量先使用字母、数字、下划线，避免 `@`、`:`、`/`、`?`、`#` 这类 URL 特殊字符。若必须使用，需要做 URL 编码。

## 4. 启动服务

在项目根目录执行：

```bash
docker compose -f docker-compose.prod.yml up -d --build
```

查看状态：

```bash
docker compose -f docker-compose.prod.yml ps
```

查看后端日志：

```bash
docker logs -f campus-backend
```

首次启动时，后端会自动执行数据库建表和初始数据插入。

## 5. 访问系统

浏览器访问：

```text
http://你的服务器IP
```

默认管理员账号：

```text
admin@mail2.sysu.edu.cn
admin123
```

首次部署后请立刻登录后台，确认管理员账号可用，并尽快修改默认密码或替换初始化账号。

## 6. 常用运维命令

更新代码并重新部署：

```bash
git pull
docker compose -f docker-compose.prod.yml up -d --build
```

重启全部服务：

```bash
docker compose -f docker-compose.prod.yml restart
```

停止服务：

```bash
docker compose -f docker-compose.prod.yml down
```

只看后端日志：

```bash
docker logs -f campus-backend
```

只看前端/Nginx日志：

```bash
docker logs -f campus-frontend
```

进入 MySQL：

```bash
docker exec -it campus-mysql mysql -u campus_user -p campus_platform
```

## 7. 数据持久化和备份

当前生产 compose 使用 Docker 命名卷保存数据：

- `mysql-data`：数据库
- `redis-data`：验证码缓存
- `uploads`：用户上传资料

备份数据库：

```bash
docker exec campus-mysql mysqldump -uroot -p campus_platform > campus_platform.sql
```

备份上传文件卷：

```bash
docker run --rm -v softengshare_uploads:/data -v "$PWD":/backup alpine tar czf /backup/uploads.tar.gz -C /data .
```

注意：不同目录名启动 Compose 时，卷名前缀可能不同。可以用下面命令查看真实卷名：

```bash
docker volume ls
```

## 8. HTTPS

如果你有域名，推荐在服务器上再加一层 Caddy 或宿主机 Nginx 做 HTTPS。

最简单的方式是先让本项目监听本机端口，例如：

```env
WEB_PORT=127.0.0.1:8081
```

然后用 Caddy 反代：

```caddyfile
你的域名 {
    reverse_proxy 127.0.0.1:8081
}
```

这种方式证书会自动申请和续期。

## 9. 部署后必须测试

至少测试这些功能：

- 注册邮箱验证码能发送，非允许邮箱域名会被拒绝。
- 普通用户登录后可以查看课程、分页搜索课程。
- 课程评价可以发布、点赞、取消点赞。
- 文件资料和链接资料上传后立即显示，后台可以隐藏/删除。
- 用户上传文件超过 `MAX_UPLOAD_MB` 时会被拒绝。
- 项目招募可以发布、申请、发布者处理申请、关闭、隐藏。
- 用户最多只能保留 5 个未关闭且未隐藏的招募。
- 管理后台用户管理、课程管理、内容管理、项目管理、今日活跃人数显示正常。
- 退出登录后访问个人中心和管理后台会被拦截。

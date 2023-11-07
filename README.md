# zero-2-prod
learning [zero to production in rust](https://www.zero2prod.com) and practice

## 技术栈
1. 数据库：postgresql
2. 类ORM：sqlx
3. 缓存：redis
4. Web框架：actix-web
5. JSON处理：serde
6. 时间处理：chrono
7. 日志：tracing
8. 参数校验：validator
9. Http请求：reqwest
10. 错误处理：thiserror
11. 部署：docker-compose

## 如何启动项目
1. 安装Docker、Rust，注意把镜像/依赖源配置为国内环境
2. 安装sqlx-cli: 在命令行执行 `cargo install sqlx-cli`
3. 启动docker: 在命令行执行 `docker compose up`
4. 生成数据库表: 在命令行执行 `sqlx migrate run`
5. 以上按顺序执行成功后，在浏览器输入`http://localhost:8000`，如果能看到`Welcome to our newsletter!`即代表项目启动成功！

数据库用户名为`postgres`，密码为`password`

网站管理员为`admin`，密码为`123456`

## 功能接口
| No | 请求方法 | 路径                     | 含义                                            |
|----|------|------------------------|-----------------------------------------------|
| 1  | GET  | /                      | 主页，如果项目成功启动，主页会显示`Welcome to our newsletter!` |
| 2  | GET  | /health_check          | 接口检查，如果前后端接口畅通，该接口应返回状态200                    |
| 3  | GET  | /login                 | 加载登录页面                                        |
| 4  | POST | /login                 | 登录                                            |
| 5  | GET  | /admin/dashboard       | 登录成功后跳转到仪表盘页面                                 |
| 6  | GET  | /admin/password        | 加载修改密码页面                                      |
| 7  | POST | /admin/password        | 修改密码                                          |
| 8  | GET  | /admin/newsletter      | 加载发布页面                                        |
| 9  | POST | /admin/newsletter      | 发布                                            |
| 10 | POST | /subscriptions         | 订阅                                            |
| 11 | GET  | /subscriptions/confirm | 确认订阅                                          |
| 12 | POST | /admin/logout          | 退出                                            |

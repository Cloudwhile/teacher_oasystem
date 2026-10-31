# TeacherOA

教师成果管理系统，用于集中填报教学、科研、专业发展、荣誉和指导学生获奖等成果，并管理相关人员、证明材料与历史记录。

## 功能

- **成果填报**：支持 15 类成果，按类型展示对应字段，可保存草稿、编辑和提交。
- **成果查询**：支持名称搜索、类型、学期、级别、状态筛选及分页，查看详情和历史修订。
- **参与人员**：记录关联教师或外部人员的姓名、角色与署名顺序。
- **证明材料**：支持 PDF、JPG、PNG 上传，以及按权限预览和下载。
- **个人档案**：查看教师工号、部门、学科、在职状态与角色信息。
- **成果审核**：审核人员查看待办、通过或填写意见退回；教师补正后可重新提交，通过的成果可手动归档。详情保留办理意见、办理人与历史修订。
- **移动端操作**：支持手机填报、材料上传及成果查看。

统计、导入导出和管理后台尚未开放。

## 环境准备

| 软件 | 版本 |
| --- | --- |
| Rust | 1.99.1 |
| Node.js | 24 |
| Bun | 1.4.4 |
| PostgreSQL | 17 |

本地运行使用 PostgreSQL 和本地文件目录。PostgreSQL 可以自行安装，也可以通过 Docker Compose 启动。

## 快速启动

以下 sh/bash 命令从仓库根目录执行。

### 1. 准备数据库

```bash
cd teacher_oa_service
docker compose -f compose.development.yml up -d postgres
```

默认连接地址为 `127.0.0.1:55432`，应用数据库为 `teacher_oa`。已有可用数据库时，可以跳过启动命令，并在 `config.development.toml` 中调整连接地址。

应用首次启动会自动执行数据库迁移。

本机已准备项目内便携 PostgreSQL 时，也可以使用：

```bash
bash scripts/start-local-postgres.sh
```

### 2. 构建界面

```bash
cd frontend-panel
bun install --frozen-lockfile
bun run build
cd ..
```

### 3. 启动应用

首次运行先在应用目录复制配置示例，按需编辑 `.env`：

```bash
cp .env.example .env
```

之后直接启动：

```bash
cargo +stable run --features openapi,metrics
```

首次运行先在 `teacher_oa_service` 目录执行 `cargo +stable run -- --init-admin`。该命令仅在没有密码账号时创建 `admin`，随机初始密码保存于私有目录 `data/private-files/initial-admin.txt`，不会打印密码。浏览器访问 **<http://127.0.0.1:3000>** 后使用该账号登录并修改密码。

管理员在“账号管理”创建正式账号、分配部门和角色、停用账号或重置密码；在“登录设置”填写企业微信企业 ID、应用 AgentId 和 Secret 并启用。将页面显示的回调地址对应域名配置到企业微信应用，随后在账号管理绑定成员 UserID。桌面端扫码、企业微信客户端内授权共用正式账号权限。Secret 加密保存且不回传；备份须包含私有目录中的 `.settings-key`。域名和 HTTPS 部署须同步更新 `auth.public_origin`、Cookie 安全配置。

默认不启用测试登录和虚构账号。旧浏览器回归用例通过测试接口装载隔离数据，需要专用测试数据库和显式启用 mock；正式登录页面始终不展示测试身份。

## 配置与数据

Windows 本机运维可使用基于 Slint 的桌面服务管理器，支持启动、停止、日志、健康检查和服务配置。构建、使用方式与管理 API 见 [桌面管理器说明](teacher_oa_service/crates/desktop_manager/README.md)。

| 位置 | 用途 |
| --- | --- |
| `teacher_oa_service/config.development.toml` | 本地运行配置 |
| `teacher_oa_service/config.example.toml` | 部署配置示例 |
| `teacher_oa_service/.env.example` | 本地配置示例 |
| `teacher_oa_service/.env` | 自动读取的本地配置 |
| `teacher_oa_service/data/private-files/` | 本地证明材料 |

程序启动时自动查找当前目录及上级目录中的 `.env`，无需手动注入环境变量。可在 `.env` 中设置 `ASTER_CONFIG`、数据库连接和企业微信密钥等配置；已有进程环境变量优先于文件中的同名配置。

本地配置使用内存会话，应用重启后需要重新登录。备份时应同时保留数据库与附件目录。部署示例仍需按实际环境配置和验收。

### 可选 Redis

默认使用内存会话，无需安装或启动 Redis。内存会话适合单实例运行，重启后需要重新登录。

需要 Redis 时，修改 `.env` 中的会话存储，并填写连接地址：

```dotenv
ASTER__AUTH__STORE=redis
AUTH_REDIS_URL=redis://127.0.0.1:56379/1
```

可以通过以下命令启动可选服务，然后重启应用：

```bash
docker compose -f compose.development.yml --profile redis up -d redis
```

## 检查与测试

在应用目录运行：

```bash
cargo +stable fmt --all --check
cargo +stable clippy --workspace --all-targets --features openapi,metrics -- -D warnings
cargo +stable test --workspace --features openapi,metrics
```

在 `teacher_oa_service/frontend-panel` 目录运行：

```bash
bun run check
bun run test
bun run build
```

启动应用后，可在同一前端目录运行浏览器测试：

```bash
bun run test:e2e
```

PostgreSQL 集成测试使用 `.env` 中 `ASTER_TEST_DATABASE_URL` 指定的独立数据库，会重建其中的数据。在应用目录运行：

```bash
cargo +stable test --features openapi,metrics --test postgres_identity -- --ignored
cargo +stable test --features openapi,metrics --test postgres_achievements -- --ignored
```

## 项目目录

```text
.
├── .github/workflows/      持续集成
├── README.md               项目入口与运行说明
└── teacher_oa_service/     应用工作区
```

应用工作区内按职责组织：

```text
teacher_oa_service/
├── src/                  服务端
├── crates/               数据库迁移
├── frontend-panel/       界面与浏览器测试
├── tests/                服务端测试
├── scripts/              本地辅助脚本
└── infra/                依赖初始化配置
```

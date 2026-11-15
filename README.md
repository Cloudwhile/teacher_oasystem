# TeacherOA · 教师教科研成果与档案管理系统

面向学校教师、教研组审核人员与系统管理员的一体化教科研成果申报、证明材料存证、分级审核归档与门户管理平台。服务端基于 **Rust（Actix Web + SeaORM + AsterForge）** 构建单二进制内嵌静态资源架构，Web 工作台采用 **React 19 + TypeScript + Vite 8**，并配套基于 **Slint** 的 **Windows 桌面运维管理器**。

`Rust 1.99.1 (Edition 2024)` · `React 19` · `TypeScript` · `Bun 1.4.4` · `PostgreSQL 17` · `Slint` · `MIT License`

---

## 目录

- [系统架构](#系统架构)
- [核心能力矩阵](#核心能力矩阵)
- [仓库目录规范](#仓库目录规范)
- [环境与工具链要求](#环境与工具链要求)
- [快速开始](#快速开始)
- [配置体系与生产安全规范](#配置体系与生产安全规范)
- [数据备份与灾难恢复](#数据备份与灾难恢复)
- [质量门禁与自动化验证](#质量门禁与自动化验证)

---

## 系统架构

系统采用前后端同源内嵌发布与严格分层鉴权设计：生产编译时将构建后的前端静态产物嵌入 `teacher_oa_service` 可执行文件；所有业务权限、数据范围与文件合法性均在服务端强制核验；桌面运维端仅通过受限本机回环协议管控自身拉起的服务子进程。

```mermaid
flowchart LR
    subgraph Clients["访问与运维终端"]
        Browser["浏览器 Web 工作台"]
        WeCom["企业微信扫码 / 客户端免登"]
        Desktop["Windows 桌面服务管理器 (Slint)"]
    end

    subgraph Service["teacher_oa_service (单二进制核心服务)"]
        Assets["内嵌前端静态资源 (rust-embed)"]
        RestAPI["业务接口 (/api/v1/* · Origin + CSRF 校验)"]
        MgmtAPI["受限本机管理接口 (/api/v1/management/*)"]
        HealthAPI["健康与指标探针 (/health · /health/ready · /health/metrics)"]
        Domain["领域服务层 (RBAC + 数据范围 · 动态 Schema · 乐观锁 · 审计)"]
    end

    subgraph Infra["持久化与外部依赖"]
        PG[("PostgreSQL 17 (业务数据与有序迁移)")]
        Storage[("私有文件目录 / S3 对象存储 (证明材料与门户资产)")]
        Redis[("Redis 7 (可选：多实例会话与登录限流)")]
    end

    Browser -->|"HTTPS / 同源会话"| Assets
    Browser -->|"REST JSON / Multipart"| RestAPI
    WeCom -->|"OAuth2 回调"| RestAPI
    Desktop -->|"127.0.0.1 + 进程级 Bearer 凭据"| MgmtAPI
    Desktop -->|"进程标准输出采集与脱敏"| Service

    RestAPI --> Domain
    MgmtAPI --> Domain
    HealthAPI --> Domain

    Domain --> PG
    Domain --> Storage
    Domain -.->|"store = redis"| Redis
```

---

## 核心能力矩阵

| 业务域 | 已实现核心能力 | 关键工程与安全机制 |
| --- | --- | --- |
| **成果全生命周期管理** | 覆盖教育教学、教育科研、专业发展、荣誉表彰、学生培养 **5 大门类共 15 类动态 Schema**（公开课、专题讲座、论文获奖、论文发表、专著、课题研究、教学成果、专利发明、教研机构成员、名师工作室、骨干教师、学科荣誉、个人先进、集体荣誉、指导学生获奖）；支持草稿暂存、未保存离开拦截、校内外相关人员署名排序与修订快照对比 | 服务端按成果 Schema 校验必填项、字典编码、金额精度与人数上限；基于版本号实现乐观锁并发冲突拦截 |
| **证明材料安全存证** | 支持拖拽与批量上传 PDF、PNG、JPEG 证明材料（单文件上限 20 MB，单条成果关联 1～20 份），提供在线预览、受权下载与我的材料库管理 | 服务端执行真实图像与受解码预算保护的 PDF 结构检验，计算并持久化 `SHA-256` 摘要，拦截被成果引用的材料误删 |
| **分级审核与归档流转** | 支持待办任务按状态与教研组筛选，执行审核通过、退回补正（强制填写退回意见）与正式归档，完整记录办理时间轴 | 服务端同时核验**操作权限码**与**数据范围**（本人 / 本教研组 / 全校），杜绝越权读取或跨范围审批 |
| **账号体系与身份认证** | 支持工号/账号密码登录、首次登录强制改密、个人资料维护，以及企业微信网页扫码与客户端内免登 | Argon2 口令哈希、复杂度校验、基于来源 IP 与账号的失败限流；写请求强制校验同源 `Origin` 与 `X-CSRF-Token` |
| **系统管理与门户定制** | 内置成员账号创建、角色与教研组分配、账号启停与口令重置；支持在线配置企业微信参数、定制登录页视觉（背景图、LOGO、站点名称、页脚）及发布多状态系统公告 | 企业微信密钥等敏感配置经本地 `.settings-key` 主密钥进行 `AES-GCM` 加密落盘；禁用账号即时吊销活动会话 |
| **Windows 桌面运维管理** | 提供独立桌面程序 `teacher_oa_desktop.exe`，支持进程启停/重启、PID 与依赖健康监控、实时日志筛选导出及核心运行参数修改 | 仅管理当前实例启动的子进程；日志流自动脱敏凭据与口令；不直连业务数据库或附件存储 |

> [!NOTE]
> 当前版本聚焦教师成果申报、材料存证、分级审核归档、系统管理与本机运维；跨维度统计报表与批量导入导出模块暂未开放。

---

## 仓库目录规范

仓库采用根级工程治理配置 + `teacher_oa_service` 核心工作区的标准分层结构：

```text
teacher_oasystem/
├── .github/workflows/ci.yml                # CI 流水线（Rust 静态检查/单测/PG 套件 + 前端构建 + E2E）
├── teacher_oa_service/                     # 核心服务工作区（Cargo Workspace + 前端工程）
│   ├── .cargo/audit.toml                   # Cargo 依赖安全审计策略
│   ├── crates/                             # 工作区独立子 Crate
│   │   ├── desktop_manager/                # Windows 桌面运维管理器（Slint UI + 子进程守护）
│   │   ├── management_protocol/            # 本机受限管理接口协议与参数校验契约
│   │   └── teacher_oa_service_migration/   # SeaORM 有序数据库迁移与 15 类成果目录种子
│   ├── frontend-panel/                     # Web 前端工程（React 19 + TypeScript + Vite 8）
│   │   ├── e2e/                            # Playwright 跨视口与真实浏览器端到端测试
│   │   ├── generated/openapi.json          # 后端导出的 OpenAPI 3 契约快照
│   │   ├── scripts/split-api.mjs           # 前端契约类型分片生成脚本
│   │   └── src/                            # 前端业务页面、通用组件、HTTP 客户端与样式体系
│   ├── infra/postgres-init/                # 开发容器测试库初始化 SQL
│   ├── scripts/                            # 本地 PostgreSQL 启动与 Windows 桌面打包脚本
│   ├── src/                                # Rust 服务端源码（api / services / domain / repositories 等）
│   ├── tests/                              # 服务端契约集成测试与 PostgreSQL 专项验证套件
│   ├── vendor/lopdf/                       # 带上传解码预算上限的本地补丁依赖
│   ├── Cargo.toml                          # Cargo 工作区清单、依赖锁定策略与构建 Profile
│   ├── Dockerfile                          # 多阶段生产容器镜像构建文件
│   ├── build.rs                            # 编译期元信息注入与前端构建产物嵌入校验
│   ├── compose.development.yml             # 本地开发依赖编排（PostgreSQL 17 / Redis / MinIO）
│   ├── config.development.toml             # 本地开发配置模板
│   ├── config.example.toml                 # 生产环境配置参考模板
│   ├── docker-compose.yml                  # 单机容器快速体验编排示例
│   └── rust-toolchain.toml                 # 固定 Rust 工具链定义（1.99.1）
├── .dockerignore                           # 根级容器构建上下文过滤规则
├── .editorconfig                           # 跨编辑器编码、缩进与换行规范
├── .gitattributes                          # Git 文本 LF 行尾归一化与二进制文件声明
├── .gitignore                              # 敏感凭据、运行时数据与构建产物忽略规则
└── README.md                               # 项目架构总览、快速启动与生产运维规范（本文档）
```

> 全栈架构、构建与运维说明统一收敛于本文档；仅随 Windows 部署包分发的桌面端使用手册独立保留于 [desktop_manager/README.md](teacher_oa_service/crates/desktop_manager/README.md)。

---

## 环境与工具链要求

| 组件 | 版本要求 | 用途说明 |
| --- | --- | --- |
| **Rust** | `1.99.1`（已由 `rust-toolchain.toml` 固定） | 编译 `teacher_oa_service`、迁移工具与 Windows 桌面管理器 |
| **Bun** | `1.4.4` | 安装前端依赖、执行类型检查、单元测试与静态构建 |
| **Node.js** | `24.x` | 运行前端 API 类型分片脚本（`split-api.mjs`）及构建工具链 |
| **PostgreSQL** | `17.x` | 生产与本地推荐主数据库（开发容器默认映射 `127.0.0.1:55432`） |
| **Redis** | `7.x`（可选） | 多实例部署时的共享认证会话与登录限流存储 |

---

## 快速开始

以下步骤均以 `teacher_oa_service` 作为工作目录执行。

### 1. 启动本地 PostgreSQL 数据库

推荐使用内置开发编排启动隔离端口（`55432`）的 PostgreSQL 17，容器首次启动会自动创建开发库 `teacher_oa` 与测试库 `teacher_oa_test`：

```bash
cd teacher_oa_service
docker compose -f compose.development.yml up -d postgres
```

若使用本机自建 PostgreSQL，请确保创建相应数据库并在后续 `.env` 中同步更新连接串。

### 2. 安装依赖并构建前端界面

Release 构建强制要求 `frontend-panel/dist` 存在真实前端产物；提前构建亦可让本地开发直接加载完整界面：

```bash
cd frontend-panel
bun install --frozen-lockfile
bun run build
cd ..
```

### 3. 配置环境变量并初始化系统管理员

复制环境变量模板并按需核对数据库连接参数：

```bash
# Linux / macOS / Git Bash
cp .env.example .env
```

```powershell
# Windows PowerShell
Copy-Item .env.example .env
```

首次启动前，显式创建初始系统管理员账号（服务会自动执行待运行的数据库迁移）：

```bash
cargo run -- --init-admin
```

命令执行成功后，终端会输出初始凭据文件路径（开发环境默认为 `data/private-files/bootstrap-admin.txt`）。

### 4. 启动服务并完成首次登录

```bash
cargo run --features openapi,metrics
```

启动后可访问以下入口：

- **Web 工作台**：`http://127.0.0.1:3000`（使用初始管理员凭据登录，按引导修改强密码后**立即删除** `bootstrap-admin.txt`）
- **存活与就绪探针**：`http://127.0.0.1:3000/health` · `http://127.0.0.1:3000/health/ready`
- **Prometheus 指标**（启用 `metrics` Feature 时）：`http://127.0.0.1:3000/health/metrics`
- **Swagger UI 与 OpenAPI 文档**（仅限 Debug 构建且启用 `openapi` Feature 时）：`http://127.0.0.1:3000/swagger-ui/` · `/api-docs/openapi.json`

> [!IMPORTANT]
> 若在 Windows 服务器上进行图形化启停、日志排查与运行参数维护，请参阅 [Windows 桌面服务管理器说明](teacher_oa_service/crates/desktop_manager/README.md) 使用 `./scripts/build-desktop.ps1` 生成独立部署包。

---

## 配置体系与生产安全规范

### 配置加载与覆盖优先级

系统启动时按以下由低到高的顺序合并配置：

1. **TOML 配置文件**：读取 `ASTER_CONFIG` 指定的路径；未设置时默认读取 `data/config.toml`（参考 [`config.example.toml`](teacher_oa_service/config.example.toml)）。配置文件内的相对路径均**相对于该配置文件所在目录**解析。
2. **`.env` 环境文件**：自动自当前工作目录向上查找 `.env`，仅补充当前进程尚未定义的环境变量。
3. **进程环境变量**：通过 `ASTER__SECTION__KEY` 双下划线命名覆盖对应 TOML 配置项（例如 `ASTER__DATABASE__URL`、`ASTER__AUTH__PUBLIC_ORIGIN`），进程已注入的环境变量优先级最高。

### 生产环境安全基线

| 检查项 | 生产环境强制要求 |
| --- | --- |
| **HTTPS 与同源配置** | 前置反向代理终止 TLS，设置 `[auth] environment = "production"`、`cookie_secure = true`，并将 `public_origin` 精确设为对外访问的 `https://域名`；严禁开启 `mock_enabled` 或 `mock_seed` |
| **会话与限流后端** | 单实例可使用 `ASTER__AUTH__STORE=memory`（需确保反代正确传递真实客户端 IP 以避免全体用户共享同一限流桶）；多实例水平扩展**必须**设为 `redis` 并配置 `AUTH_REDIS_URL`，Redis 故障时系统拒绝静默降级至内存 |
| **文件与对象存储** | 单实例可将 `[storage]` 指向非公开本地目录；多实例部署**必须**统一配置 S3 兼容对象存储，确保各节点共享证明材料与门户图片 |
| **敏感设置主密钥** | 管理后台保存的企业微信密钥等敏感项由存储根目录下的 `.settings-key` 加密保护；严禁提交至代码库，亦严禁在运行期删除或覆盖 |
| **企业微信回调域名** | 配置 `ASTER__AUTH__WECOM_CORP_ID`、`ASTER__AUTH__WECOM_AGENT_ID`、`WECOM_ALLOWED_CORP_ID`、`WECOM_CORP_SECRET` 及 `https://域名/api/v1/auth/wecom/callback`，并在企业微信后台登记可信域名 |

---

## 数据备份与灾难恢复

由于业务状态同时跨越关系型数据库、证明材料存储与加密设置主密钥，生产备份必须保证三者处于**一致时间点**。

### 1. 必选备份清单

1. **运行配置**：`config.toml` 与外部环境变量编排清单（脱敏归档或加密保管）。
2. **PostgreSQL 数据库**：包含账号权限、成果记录、修订快照、审核轨迹与文件元数据（含 `SHA-256` 校验值）。
3. **附件与门户资源**：本地存储目录（如 `data/private-files`）或 S3 存储桶内的证明材料、登录页图片及关联资源。
4. **设置加密密钥**：附件存储根目录下的隐藏密钥文件 `.settings-key`（**丢失将导致数据库内加密配置无法解密**）。

### 2. 标准备份流程

1. 选择业务低峰维护窗口，暂停外部写入或优雅停止 `teacher_oa_service`，确保数据库元数据与底层文件对象完全对齐。
2. 执行 PostgreSQL 一致性逻辑备份：
   ```bash
   pg_dump --format=custom --no-owner --file=teacheroa-$(date +%F).dump "$DATABASE_URL"
   ```
3. 完整打包存储目录（须显式包含隐藏文件 `.settings-key`）或同步 S3 版本化快照：
   ```bash
   tar -czf teacheroa-files-$(date +%F).tar.gz -C data private-files config.toml
   ```
4. 将备份产物异机加密存储，并记录本次备份对应的服务版本号与文件校验和。

### 3. 恢复与完整性校验

1. 在隔离恢复环境还原 `config.toml`、附件存储及**原 `.settings-key` 文件**（确认运行账户具备最小读写权限）。
2. 创建空目标库并导入备份：
   ```bash
   pg_restore --clean --if-exists --no-owner --dbname="$RESTORE_DATABASE_URL" teacheroa-YYYY-MM-DD.dump
   ```
3. 启动对应版本的 `teacher_oa_service`，确认 `/health/ready` 返回 `200 OK`（`status: "ready"`）。
4. 随机抽检已归档成果的证明材料下载与在线预览，核对底层文件计算的 `SHA-256` 与数据库记录一致，并验证管理后台加密配置读取正常后再切换生产流量。

---

## 质量门禁与自动化验证

仓库在 [`.github/workflows/ci.yml`](.github/workflows/ci.yml) 中执行全链路门禁验证。本地提交前可在 `teacher_oa_service` 目录按需运行：

### Rust 服务端与工作区检查

```bash
# 格式检查与 Clippy 零警告静态分析
cargo fmt --all --check
cargo clippy --workspace --all-targets --features openapi,metrics -- -D warnings

# 工作区单元与默认集成测试
cargo test --workspace --features openapi,metrics

# PostgreSQL 专项集成测试（会重建 ASTER_TEST_DATABASE_URL 指向的独立测试库，切勿指向生产库）
ASTER_TEST_DATABASE_URL=postgres://teacher_oa:teacher_oa_local@127.0.0.1:55432/teacher_oa_test \
  cargo test --features openapi --test postgres_identity -- --ignored
ASTER_TEST_DATABASE_URL=postgres://teacher_oa:teacher_oa_local@127.0.0.1:55432/teacher_oa_test \
  cargo test --features openapi --test postgres_achievements -- --ignored
```

### OpenAPI 契约同步与 Web 前端验证

> **编译特性与迁移约定**：服务端默认 `default = []`，按需开启 `openapi`（仅 Debug 构建暴露契约文档并用于生成快照）、`metrics`（暴露 `/health/metrics`）或 `jemalloc`；数据库迁移在启动时自动运行，亦可执行 `cargo run -p teacher_oa_service_migration -- status|up` 手动检查或升级；修改 API 后严禁手改前端生成类型，须按下方流程重新导出。

```bash
# 1. 在 teacher_oa_service 目录从后端路由生成最新 openapi.json
cargo test --features openapi --test generate_openapi

# 2. 在 teacher_oa_service/frontend-panel 目录同步 TypeScript 类型并运行检查与测试
cd frontend-panel
bun run generate-api
bun run check
bun run test
bun run build

# 3. 端到端浏览器回归测试（需先在 3000 端口启动开发态服务或指定 TEACHEROA_BASE_URL）
bun run test:e2e
```

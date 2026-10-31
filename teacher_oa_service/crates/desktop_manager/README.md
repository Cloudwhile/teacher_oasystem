# TeacherOA Windows 服务管理器

基于 Rust 和 Slint 的 Windows 桌面管理程序。管理器不自动启动服务；首次打开后，在“配置管理”选择服务程序、工作目录、现有 TOML 配置和本机服务地址，保存后启动。

## 构建与运行

在 `teacher_oa_service` 目录执行：

```powershell
cargo build -p teacher_oa_service -p teacher_oa_desktop
./target/debug/teacher_oa_desktop.exe
```

发布打包：

```powershell
./scripts/build-desktop.ps1
```

脚本先构建 Web 界面，再编译 Windows 程序。`target/desktop-package/` 包含两个 EXE 和配置示例。首次使用时复制示例并填写部署配置，管理器启动设置保存在 `%LOCALAPPDATA%/TeacherOA/desktop-manager.json`。服务配置文件的相对路径相对于该文件所在目录；服务启动时仍按现有机制加载工作目录及上级目录中的 `.env`。

## 功能

- 运行概览：启动、优雅停止、重启、PID、运行时长、服务版本和数据库/缓存/会话健康状态。
- 运行日志：捕获当前子进程的标准输出和错误，筛选、暂停、跟随、导出和清空视图；最多保留1500条、约512KiB内容，单条分块上限8KiB。清空视图不会删除服务日志文件。
- 启动设置：选择 EXE、工作目录、TOML 文件和服务地址；运行时禁止切换管理目标。
- 服务配置：通过管理 API 修改监听 IP、端口、工作线程、连接池大小、日志级别与格式；保存后在下次启动生效。修改端口后，停止时同步更新启动地址。
- 关闭窗口：运行时确认停止并退出；连接异常时保留窗口供排查，可单独确认强制结束当前子进程。
- “打开后台”使用浏览器进入既有账号登录流程，业务数据管理保持原有权限与 CSRF 验证。

该管理器只管理自己启动的子进程，不接管端口上已有服务，不通过进程名称批量终止程序。桌面端没有数据库驱动、SQL、数据库文件读写或迁移逻辑；数据库连接与检查均由服务执行。

## 本机管理 API

请求、响应类型定义于 `../management_protocol/src/lib.rs`，采用服务已有的 `{ "data": ... }` 响应与错误格式。

| 方法 | 路径 | 请求 / 响应 |
| --- | --- | --- |
| GET | `/api/v1/management/status` | `ServiceStatus`：服务名、版本、PID、运行秒数、停止状态、依赖检查结果 |
| GET | `/api/v1/management/config` | `ConfigSnapshot`：生效配置、文件配置、修订摘要、是否需重启 |
| PUT | `/api/v1/management/config` | `SaveConfig { config, revision }` → `ConfigSnapshot` |
| POST | `/api/v1/management/shutdown` | `{ "stopping": true }`，随后取消运行时关闭令牌并等待资源退出 |

普通启动不启用这些操作，返回404。管理器每次启动子进程生成32字节随机凭据，通过仅该子进程的 `TEACHER_OA_MANAGEMENT_TOKEN` 环境变量传入；请求须携带 `Authorization: Bearer <凭据>`。服务检查真实连接来源为回环地址，拒绝带 `Origin` 或 `Sec-Fetch-Site` 的浏览器请求；管理器只连接字面量回环 IP，禁用代理及跳转，并核对响应 PID。凭据不写入启动设置、不进入命令行、不授予任何业务 API 权限。

配置接口只允许上述六类运行参数；不返回数据库 URL、密码或业务记录。监听 IP 限本机或全部网卡，端口1～65535，工作线程0～128，连接池1～256，日志级别 `trace/debug/info/warn/error`，格式 `text/json`。环境变量覆盖相关配置时拒绝保存，避免显示成功但重启不生效。写入前检查修订摘要，原子替换配置文件并保留其他 TOML 值与文件权限；TOML 注释会被规范化输出移除。

账号、企业微信、登录页及公告继续使用现有 `/api/v1/admin/...` API。新增业务管理能力也必须在服务 API 中实现并鉴权，桌面客户端不得直连数据库或访问业务存储。

## 验证

```powershell
cargo test -p teacher_oa_desktop --lib
cargo test -p teacher_oa_service --test management
$env:TEACHER_OA_TEST_SERVICE = (Resolve-Path ./target/debug/teacher_oa_service.exe).Path
cargo test -p teacher_oa_desktop --test lifecycle -- --ignored
```

生命周期测试在系统临时目录生成独立 SQLite 配置和内存附件存储，不读取项目 `.env`，不连接部署数据库；验证真实进程的启动、重复启动拒绝、状态、配置保存、端口变更、重启和优雅退出。

可用 `teacher_oa_desktop.exe --screenshot <绝对PNG路径> [0|1|2]` 生成实际 Slint 窗口截图，依次对应概览、日志与配置页面；此模式不启动后端。

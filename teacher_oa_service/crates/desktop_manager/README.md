# TeacherOA Windows 服务管理器 (`teacher_oa_desktop`)

基于 **Rust + Slint** 构建的 Windows 原生桌面运维管理程序，用于在本机安全地启停 `teacher_oa_service`、观测依赖健康度、排查脱敏运行日志及维护基础服务配置。

项目整体架构与服务端部署要求请参阅仓库根目录 [README.md](../../../README.md)。管理器打开后不会自动拉起服务；首次使用时请在「配置管理」中选择服务可执行文件、工作目录、现有 TOML 配置文件与本机服务地址，保存后再点击启动。

---

## 构建与打包

在 `teacher_oa_service` 目录执行开发构建：

```powershell
cargo build -p teacher_oa_service -p teacher_oa_desktop
./target/debug/teacher_oa_desktop.exe
```

生成独立 Windows 部署目录（自动执行前端 `bun run build`、Release 编译并汇总双 EXE、配置模板与说明文档至 `target/desktop-package/`）：

```powershell
./scripts/build-desktop.ps1
```

管理器自身的窗口与路径偏好保存在 `%LOCALAPPDATA%/TeacherOA/desktop-manager.json`。服务 TOML 配置内的相对路径均相对于该配置文件所在目录解析；服务子进程启动时仍按标准规则读取工作目录及上级目录中的 `.env`。

---

## 核心功能与进程边界

- **运行概览**：执行启动、优雅停止与重启，实时展示进程 PID、运行时长、服务版本以及数据库、缓存、会话存储组件的健康检查结果。
- **运行日志（自动脱敏）**：实时采集当前受管子进程的标准输出与标准错误，支持级别/关键字筛选、暂停跟随、导出与清空视图（内存环形缓冲上限 1500 条、约 512 KiB，单分块上限 8 KiB；采集与导出前自动脱敏 `Bearer` 令牌、口令、密钥、Cookie 与数据库连接凭据；清空视图不会删除磁盘日志文件）。
- **启动设置**：配置服务 EXE、工作目录、TOML 文件路径与回环访问地址；服务运行期间锁定目标切换，防止误操作失联。
- **服务配置**：通过本机管理 API 原子更新监听 IP、端口、工作线程、连接池大小、日志级别与日志格式；保存后于下次重启生效，若修改端口则在停止后自动同步本机探针地址。
- **安全退出保护**：关闭窗口时若服务仍在运行会提示优雅停止并退出；若连接异常则保留窗口供排障，并支持二次确认后仅强制终止当前受管子进程。
- **业务后台跳转**：「打开后台」调用系统默认浏览器进入标准 Web 登录流程，所有业务管理继续受服务端 RBAC、数据范围与 CSRF 校验约束。

> **隔离原则**：管理器仅管控当前实例亲手拉起的子进程，绝不接管端口上已有的外部服务，亦不按进程名批量杀进程；桌面端不包含任何数据库驱动、SQL 读写或业务存储访问逻辑。

---

## 本机受限管理 API

共享数据结构定义于 [`../management_protocol/src/lib.rs`](../management_protocol/src/lib.rs)，复用服务端统一的 `{ "data": ... }` 响应与错误信封：

| 方法 | 路径 | 请求 / 响应说明 |
| --- | --- | --- |
| `GET` | `/api/v1/management/status` | 返回 `ServiceStatus`（服务名、版本、PID、运行秒数、停止状态与依赖健康列表） |
| `GET` | `/api/v1/management/config` | 返回 `ConfigSnapshot`（当前生效配置、文件保存配置、修订摘要与是否需重启） |
| `PUT` | `/api/v1/management/config` | 提交 `SaveConfig { config, revision }`，校验通过后原子写回配置文件并返回最新 `ConfigSnapshot` |
| `POST` | `/api/v1/management/shutdown` | 返回 `{ "stopping": true }`，随后触发服务端关闭令牌并等待连接优雅排空 |

### 鉴权与防篡改约束

1. **进程级临时凭据**：常规启动不注册上述接口（返回 `404`）。管理器每次拉起子进程时生成 32 字节密码学随机凭据，仅通过该子进程独占的 `TEACHER_OA_MANAGEMENT_TOKEN` 环境变量注入，并以 `Authorization: Bearer <凭据>` 通信；凭据不落盘、不入命令行参数，且不具备任何业务 `/api/v1/*` 权限。
2. **严格回环与反跨站**：服务端强制核验真实 TCP 对端为回环地址，并拒绝携带 `Origin` 或 `Sec-Fetch-Site` 的浏览器请求；桌面客户端仅连接字面量回环 IP、禁用系统代理与重定向，并严格核对响应中的子进程 `pid`。
3. **受控参数范围**：仅允许读写监听 IP、端口（`1..=65535`）、工作线程（`0..=128`）、连接池（`1..=256`）、日志级别（`trace/debug/info/warn/error`）与格式（`text/json`）。若检测到对应配置已被环境变量覆盖则拒绝保存，避免出现“保存成功但重启不生效”的假象；写回前核对修订摘要以防并发覆盖（规范化输出会移除原 TOML 注释）。

---

## 自动化验证与界面快照

在 `teacher_oa_service` 目录执行：

```powershell
cargo test -p teacher_oa_desktop --lib
cargo test -p teacher_oa_service --test management
$env:TEACHER_OA_TEST_SERVICE = (Resolve-Path ./target/debug/teacher_oa_service.exe).Path
cargo test -p teacher_oa_desktop --test lifecycle -- --ignored
```

生命周期测试会在系统临时目录创建独立 SQLite 配置与内存附件存储（不读取项目 `.env`，亦不触碰部署数据库），完整验证真实子进程启动、重复启动拦截、健康查询、配置保存、端口迁移、重启与优雅退出。

此外，可执行 `teacher_oa_desktop.exe --screenshot <绝对PNG路径> [0|1|2]` 在不启动后端服务的前提下离线渲染概览（`0`）、日志（`1`）或配置（`2`）页面的真实 Slint 窗口截图。

use crate::{
    api::ManagementApi,
    logs::{self, SharedLogs},
    settings::LauncherSettings,
};
use management_protocol::{ConfigSnapshot, SaveConfig, ServiceStatus, TOKEN_ENV};
use std::{
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

pub struct Supervisor {
    child: Option<Child>,
    api: Option<ManagementApi>,
    pub logs: SharedLogs,
    pub starting: Option<Instant>,
    pub stopping: Option<Instant>,
    pub connected: bool,
    pub last_status: Option<ServiceStatus>,
    pub settings: LauncherSettings,
}

impl Supervisor {
    pub fn new(settings: LauncherSettings) -> Self {
        Self {
            child: None,
            api: None,
            logs: Default::default(),
            starting: None,
            stopping: None,
            connected: false,
            last_status: None,
            settings,
        }
    }
    pub fn pid(&self) -> Option<u32> {
        self.child.as_ref().map(Child::id)
    }
    pub fn start(&mut self) -> Result<(), String> {
        if self.child.is_some() {
            return Err("服务已启动，请勿重复启动".into());
        }
        self.settings.validate()?;
        let url = crate::settings::validate_base_url(&self.settings.base_url)?;
        let address = url
            .socket_addrs(|| Some(80))
            .map_err(|_| "服务地址无效")?
            .into_iter()
            .next()
            .ok_or("服务地址无效")?;
        if std::net::TcpStream::connect_timeout(&address, Duration::from_millis(300)).is_ok() {
            return Err("该端口已被占用，请关闭已有服务或更换启动地址".into());
        }
        let token: String = rand::random::<[u8; 32]>()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        let api = ManagementApi::new(&self.settings.base_url, token.clone())?;
        let mut command = Command::new(&self.settings.executable);
        command
            .current_dir(&self.settings.working_directory)
            .env("ASTER_CONFIG", &self.settings.config_path)
            .env(TOKEN_ENV, &token)
            .env("NO_COLOR", "1")
            .env("TERM", "dumb")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }
        let mut child = command
            .spawn()
            .map_err(|_| "无法启动服务程序，请检查路径和权限")?;
        if let Some(output) = child.stdout.take() {
            logs::capture(output, self.logs.clone(), token.clone());
        }
        if let Some(output) = child.stderr.take() {
            logs::capture(output, self.logs.clone(), token);
        }
        logs::record(&self.logs, &format!("服务进程已启动，PID {}", child.id()));
        self.child = Some(child);
        self.api = Some(api);
        self.starting = Some(Instant::now());
        self.stopping = None;
        self.connected = false;
        self.last_status = None;
        Ok(())
    }
    pub fn poll(&mut self) -> Result<(), String> {
        if let Some(child) = self.child.as_mut()
            && let Some(exit) = child.try_wait().map_err(|_| "无法读取进程状态")?
        {
            let expected = self.stopping.is_some();
            logs::record(&self.logs, &format!("服务进程已退出：{exit}"));
            self.child = None;
            self.api = None;
            self.starting = None;
            self.stopping = None;
            self.connected = false;
            self.last_status = None;
            return if expected && exit.success() {
                Ok(())
            } else {
                Err(format!("服务已退出（{exit}），请查看日志"))
            };
        }
        if self.child.is_none() || self.stopping.is_some() {
            return Ok(());
        }
        match self.api.as_ref().ok_or("管理连接尚未建立")?.status() {
            Ok(status) => {
                if Some(status.pid) != self.pid() || status.service != "teacher_oa_service" {
                    self.connected = false;
                    return Err("服务身份不匹配，已停止发送管理操作".into());
                }
                self.connected = true;
                self.starting = None;
                self.last_status = Some(status);
                Ok(())
            }
            Err(error) => {
                self.connected = false;
                if self
                    .starting
                    .is_some_and(|started| started.elapsed() < Duration::from_secs(45))
                {
                    Ok(())
                } else {
                    Err(error)
                }
            }
        }
    }
    fn verified_api(&self) -> Result<&ManagementApi, String> {
        if self.pid().is_none() || !self.connected || self.stopping.is_some() {
            return Err("请等待当前服务的管理连接就绪".into());
        }
        self.api.as_ref().ok_or_else(|| "管理连接尚未建立".into())
    }
    pub fn config(&self) -> Result<ConfigSnapshot, String> {
        self.verified_api()?.config()
    }
    pub fn save_config(&self, value: &SaveConfig) -> Result<ConfigSnapshot, String> {
        value.config.validate().map_err(String::from)?;
        self.verified_api()?.save_config(value)
    }
    pub fn stop(&mut self) -> Result<(), String> {
        self.verified_api()?.shutdown()?;
        self.stopping = Some(Instant::now());
        logs::record(&self.logs, "已请求停止服务，正在等待请求处理和日志写入完成");
        Ok(())
    }
    /// Only called after a separate explicit user confirmation.
    pub fn force_stop(&mut self) -> Result<(), String> {
        if let Some(child) = self.child.as_mut() {
            child.kill().map_err(|_| "无法结束服务进程")?;
            child.wait().map_err(|_| "无法回收服务进程")?;
        }
        self.child = None;
        self.api = None;
        self.starting = None;
        self.stopping = None;
        self.connected = false;
        self.last_status = None;
        logs::record(&self.logs, "已结束当前管理器启动的服务进程");
        Ok(())
    }
}

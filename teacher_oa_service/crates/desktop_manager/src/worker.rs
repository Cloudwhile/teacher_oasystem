use crate::{
    logs,
    process::Supervisor,
    settings::{self, LauncherSettings},
};
use management_protocol::{ConfigSnapshot, SaveConfig};
use std::{
    path::PathBuf,
    sync::mpsc::{Receiver, SyncSender, TrySendError},
    time::{Duration, Instant},
};

pub enum Command {
    Start(LauncherSettings),
    Stop,
    Restart,
    ForceStop,
    Refresh,
    ReadConfig,
    SaveConfig(SaveConfig),
    SaveLauncher(LauncherSettings),
    ClearLogs,
    ExportLogs(PathBuf),
    Exit,
}

pub enum Event {
    View(View),
    Logs(String),
    Config(ConfigSnapshot),
    Feedback(String, bool),
    LauncherSaved(LauncherSettings),
    Quit,
}

pub struct View {
    pub status: String,
    pub color: (u8, u8, u8),
    pub pid: String,
    pub uptime: String,
    pub dependencies: String,
    pub version: String,
    pub running: bool,
    pub connected: bool,
    pub can_force: bool,
}

fn feedback(events: &SyncSender<Event>, result: Result<(), String>, success: &str) {
    let _ = events.send(match result {
        Ok(()) => Event::Feedback(success.into(), false),
        Err(error) => Event::Feedback(error, true),
    });
}

pub fn run(settings: LauncherSettings, commands: Receiver<Command>, events: SyncSender<Event>) {
    let mut supervisor = Supervisor::new(settings);
    let mut next_poll = Instant::now();
    let mut generation = u64::MAX;
    let mut restart = false;
    let mut exiting = false;
    let mut next_base_url = None;
    let mut was_running = false;
    loop {
        match commands.recv_timeout(Duration::from_millis(200)) {
            Ok(command) => {
                match command {
                    Command::Start(settings) => {
                        supervisor.settings = settings;
                        feedback(&events, supervisor.start(), "正在启动服务");
                    }
                    Command::Stop | Command::Restart | Command::Exit => {
                        restart = matches!(command, Command::Restart);
                        exiting = matches!(command, Command::Exit);
                        if supervisor.pid().is_none() {
                            if exiting {
                                let _ = events.send(Event::Quit);
                                return;
                            }
                        } else if let Err(error) = supervisor.stop() {
                            restart = false;
                            exiting = false;
                            feedback(&events, Err(error), "");
                        } else {
                            feedback(&events, Ok(()), "正在等待服务停止");
                        }
                    }
                    Command::ForceStop => {
                        restart = false;
                        feedback(&events, supervisor.force_stop(), "服务已结束");
                    }
                    Command::Refresh => {}
                    Command::ReadConfig => match supervisor.config() {
                        Ok(snapshot) => {
                            let _ = events.send(Event::Config(snapshot));
                            feedback(&events, Ok(()), "已读取服务配置");
                        }
                        Err(error) => feedback(&events, Err(error), ""),
                    },
                    Command::SaveConfig(config) => match supervisor.save_config(&config) {
                        Ok(snapshot) => {
                            if let Ok(ip) = snapshot.saved.host.parse::<std::net::IpAddr>() {
                                let ip = if ip.is_loopback() {
                                    ip
                                } else if ip.is_ipv6() {
                                    std::net::IpAddr::V6(std::net::Ipv6Addr::LOCALHOST)
                                } else {
                                    std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)
                                };
                                next_base_url = Some(format!(
                                    "http://{}",
                                    std::net::SocketAddr::new(ip, snapshot.saved.port)
                                ));
                            }
                            let _ = events.send(Event::Config(snapshot));
                            feedback(&events, Ok(()), "配置已保存，下次启动生效");
                        }
                        Err(error) => feedback(&events, Err(error), ""),
                    },
                    Command::SaveLauncher(settings) => {
                        if supervisor.pid().is_some() {
                            feedback(&events, Err("请先停止服务再修改启动设置".into()), "");
                        } else {
                            match settings::save(&settings) {
                                Ok(()) => {
                                    supervisor.settings = settings.clone();
                                    let _ = events.send(Event::LauncherSaved(settings));
                                    feedback(&events, Ok(()), "启动设置已保存");
                                }
                                Err(error) => feedback(&events, Err(error), ""),
                            }
                        }
                    }
                    Command::ClearLogs => {
                        if let Ok(mut logs) = supervisor.logs.lock() {
                            logs.clear();
                        }
                    }
                    Command::ExportLogs(path) => {
                        let result = supervisor
                            .logs
                            .lock()
                            .map_err(|_| "无法读取日志".into())
                            .and_then(|logs| {
                                std::fs::write(path, logs.text(""))
                                    .map_err(|_| "无法导出日志，请检查路径和权限".into())
                            });
                        feedback(&events, result, "日志已导出");
                    }
                }
                // Every command completes on this thread; the UI always receives a completion event.
                let _ = events.send(Event::View(view(&supervisor)));
                next_poll = Instant::now();
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                // Event loop closed unexpectedly. Ask only our own child to stop, without force.
                if supervisor.connected {
                    let _ = supervisor.stop();
                }
                return;
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
        }
        if Instant::now() >= next_poll {
            let previously_connected = supervisor.connected;
            if let Err(error) = supervisor.poll() {
                feedback(&events, Err(error), "");
            }
            if !previously_connected && supervisor.connected {
                logs::record(&supervisor.logs, "管理连接已就绪");
            }
            if was_running && supervisor.pid().is_none() {
                if let Some(base_url) = next_base_url.take() {
                    supervisor.settings.base_url = base_url;
                    match settings::save(&supervisor.settings) {
                        Ok(()) => {
                            let _ = events.send(Event::LauncherSaved(supervisor.settings.clone()));
                        }
                        Err(error) => {
                            feedback(&events, Err(error), "");
                            restart = false;
                        }
                    }
                }
                if exiting {
                    let _ = events.send(Event::Quit);
                    return;
                }
                if restart {
                    restart = false;
                    feedback(&events, supervisor.start(), "正在重新启动服务");
                }
            }
            was_running = supervisor.pid().is_some();
            let _ = events.send(Event::View(view(&supervisor)));
            next_poll = Instant::now()
                + Duration::from_secs(if supervisor.stopping.is_some() { 1 } else { 3 });
        }
        if let Ok(logs) = supervisor.logs.lock()
            && logs.generation() != generation
        {
            // Native dialogs can pause UI delivery. Keep queued log snapshots bounded as well.
            match events.try_send(Event::Logs(logs.text(""))) {
                Ok(()) => generation = logs.generation(),
                Err(TrySendError::Full(_)) => {}
                Err(TrySendError::Disconnected(_)) => return,
            }
        }
    }
}

fn view(supervisor: &Supervisor) -> View {
    let running = supervisor.pid().is_some();
    let connected = supervisor.connected && supervisor.stopping.is_none();
    let healthy = supervisor
        .last_status
        .as_ref()
        .is_some_and(|status| status.dependencies.iter().all(|row| row.healthy));
    let (status, color) = if !running {
        ("未启动", (113, 128, 150))
    } else if supervisor.stopping.is_some() {
        ("正在停止", (166, 107, 24))
    } else if connected && healthy {
        ("运行正常", (29, 122, 86))
    } else if connected {
        ("依赖异常", (180, 62, 46))
    } else if supervisor
        .starting
        .is_some_and(|started| started.elapsed() < Duration::from_secs(45))
    {
        ("正在启动", (31, 100, 202))
    } else {
        ("连接异常", (180, 62, 46))
    };
    View {
        status: status.into(),
        color,
        pid: supervisor
            .pid()
            .map(|pid| pid.to_string())
            .unwrap_or_else(|| "—".into()),
        uptime: supervisor
            .last_status
            .as_ref()
            .map(|status| {
                let seconds = status.uptime_seconds;
                format!(
                    "{:02}:{:02}:{:02}",
                    seconds / 3600,
                    seconds / 60 % 60,
                    seconds % 60
                )
            })
            .unwrap_or_else(|| "—".into()),
        dependencies: if connected {
            supervisor
                .last_status
                .as_ref()
                .map(|status| {
                    status
                        .dependencies
                        .iter()
                        .map(|row| {
                            format!(
                                "{}  {}",
                                row.name,
                                if row.healthy { "正常" } else { "异常" }
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("     ·     ")
                })
                .unwrap_or_default()
        } else {
            "等待服务连接".into()
        },
        version: supervisor
            .last_status
            .as_ref()
            .map(|status| status.version.clone())
            .unwrap_or_else(|| "—".into()),
        running,
        connected,
        can_force: running
            && (!connected
                || supervisor
                    .stopping
                    .is_some_and(|started| started.elapsed() > Duration::from_secs(20))),
    }
}

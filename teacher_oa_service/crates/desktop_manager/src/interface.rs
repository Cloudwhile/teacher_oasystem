use crate::AppWindow;
use management_protocol::{ConfigSnapshot, SaveConfig, ServiceConfig};
use slint::ComponentHandle;
use std::{
    cell::RefCell,
    rc::Rc,
    sync::mpsc::{Receiver, Sender},
    time::Duration,
};
use teacher_oa_desktop::{
    settings::LauncherSettings,
    worker::{Command, Event},
};

pub fn show_feedback(ui: &AppWindow, text: &str, error: bool) {
    ui.set_feedback(text.into());
    ui.set_feedback_error(error);
}

pub fn set_launcher(ui: &AppWindow, settings: &LauncherSettings) {
    ui.set_executable(settings.executable.clone().into());
    ui.set_working_directory(settings.working_directory.clone().into());
    ui.set_config_path(settings.config_path.clone().into());
    ui.set_base_url(settings.base_url.clone().into());
    ui.set_launcher_dirty(false);
}

fn launcher(ui: &AppWindow) -> LauncherSettings {
    LauncherSettings {
        executable: ui.get_executable().to_string(),
        working_directory: ui.get_working_directory().to_string(),
        config_path: ui.get_config_path().to_string(),
        base_url: ui.get_base_url().to_string(),
    }
}

fn set_config(ui: &AppWindow, snapshot: &ConfigSnapshot) {
    ui.set_host(snapshot.saved.host.clone().into());
    ui.set_port(snapshot.saved.port.to_string().into());
    ui.set_workers(snapshot.saved.workers.to_string().into());
    ui.set_pool_size(snapshot.saved.pool_size.to_string().into());
    ui.set_log_level_index(
        ["trace", "debug", "info", "warn", "error"]
            .iter()
            .position(|level| *level == snapshot.saved.log_level)
            .and_then(|i| i32::try_from(i).ok())
            .unwrap_or(2),
    );
    ui.set_log_format_index(i32::from(snapshot.saved.log_format == "json"));
    ui.set_restart_required(snapshot.restart_required);
    ui.set_config_loaded(true);
    ui.set_config_dirty(false);
}

fn confirm(kind: i32, running: bool) -> bool {
    let (title, description, action) = match kind {
        1 => ("停止服务？", "系统将等待当前请求处理完成。", "停止"),
        2 => (
            "重启服务？",
            "系统将停止当前服务并使用已保存的配置重新启动。",
            "重启",
        ),
        3 => ("强制结束服务？", "尚未完成的请求可能中断。", "强制结束"),
        4 => (
            "退出管理器？",
            if running {
                "将停止服务后退出。未保存的修改会被放弃。"
            } else {
                "未保存的修改会被放弃。"
            },
            if running { "停止并退出" } else { "退出" },
        ),
        5 => ("重新读取配置？", "尚未保存的服务配置将被放弃。", "重新读取"),
        _ => return false,
    };
    rfd::MessageDialog::new()
        .set_title(title)
        .set_description(description)
        .set_level(if kind == 3 {
            rfd::MessageLevel::Warning
        } else {
            rfd::MessageLevel::Info
        })
        .set_buttons(rfd::MessageButtons::OkCancelCustom(
            action.into(),
            "取消".into(),
        ))
        .show()
        == rfd::MessageDialogResult::Custom(action.into())
}

fn send(ui: &AppWindow, sender: &Sender<Command>, command: Command) {
    if ui.get_busy() {
        return;
    }
    ui.set_busy(true);
    if sender.send(command).is_err() {
        ui.set_busy(false);
        show_feedback(ui, "管理线程已退出，请重新打开管理器", true);
    }
}

pub fn bind(ui: &AppWindow, commands: Sender<Command>, responses: Receiver<Event>) {
    let snapshot: Rc<RefCell<Option<ConfigSnapshot>>> = Default::default();
    let log_text = Rc::new(RefCell::new(String::new()));
    let weak = ui.as_weak();
    let sender = commands.clone();
    ui.on_start_service(move || {
        if let Some(ui) = weak.upgrade() {
            if ui.get_launcher_dirty() {
                show_feedback(&ui, "请先保存启动设置", true);
                ui.set_page(2);
            } else {
                send(&ui, &sender, Command::Start(launcher(&ui)));
            }
        }
    });
    let weak = ui.as_weak();
    let sender = commands.clone();
    ui.on_save_launcher(move || {
        if let Some(ui) = weak.upgrade() {
            send(&ui, &sender, Command::SaveLauncher(launcher(&ui)));
        }
    });
    let weak = ui.as_weak();
    let sender = commands.clone();
    ui.on_read_config(move || {
        if let Some(ui) = weak.upgrade() {
            send(&ui, &sender, Command::ReadConfig);
        }
    });
    let weak = ui.as_weak();
    let sender = commands.clone();
    let config = snapshot.clone();
    ui.on_save_config(move || {
        if let Some(ui) = weak.upgrade() {
            let config = config.borrow();
            let Some(snapshot) = config.as_ref() else {
                return;
            };
            let parse = || -> Result<SaveConfig, String> {
                let value = ServiceConfig {
                    host: ui.get_host().trim().into(),
                    port: ui.get_port().parse().map_err(|_| "监听端口无效")?,
                    workers: ui.get_workers().parse().map_err(|_| "工作线程数量无效")?,
                    pool_size: ui.get_pool_size().parse().map_err(|_| "连接池数量无效")?,
                    log_level: ["trace", "debug", "info", "warn", "error"]
                        .get(usize::try_from(ui.get_log_level_index()).unwrap_or(2))
                        .unwrap_or(&"info")
                        .to_string(),
                    log_format: if ui.get_log_format_index() == 1 {
                        "json"
                    } else {
                        "text"
                    }
                    .into(),
                };
                value.validate().map_err(String::from)?;
                Ok(SaveConfig {
                    config: value,
                    revision: snapshot.revision.clone(),
                })
            };
            match parse() {
                Ok(value) => send(&ui, &sender, Command::SaveConfig(value)),
                Err(error) => show_feedback(&ui, &error, true),
            }
        }
    });
    let weak = ui.as_weak();
    let sender = commands.clone();
    ui.on_confirm_action(move |kind| {
        if let Some(ui) = weak.upgrade()
            && !ui.get_busy()
            && confirm(kind, ui.get_running())
        {
            send(
                &ui,
                &sender,
                match kind {
                    1 => Command::Stop,
                    2 => Command::Restart,
                    3 => Command::ForceStop,
                    5 => Command::ReadConfig,
                    _ => return,
                },
            );
        }
    });
    let weak = ui.as_weak();
    let sender = commands.clone();
    ui.on_refresh_status(move || {
        if let Some(ui) = weak.upgrade() {
            send(&ui, &sender, Command::Refresh);
        }
    });
    let weak = ui.as_weak();
    let sender = commands.clone();
    ui.on_clear_logs(move || {
        if let Some(ui) = weak.upgrade() {
            let _ = sender.send(Command::ClearLogs);
            ui.set_logs("".into());
        }
    });
    let weak = ui.as_weak();
    let sender = commands.clone();
    ui.on_export_logs(move || {
        if let Some(ui) = weak.upgrade()
            && let Some(path) = rfd::FileDialog::new()
                .set_title("导出运行日志")
                .set_file_name("teacher-oa.log")
                .add_filter("日志", &["log"])
                .save_file()
        {
            send(&ui, &sender, Command::ExportLogs(path));
        }
    });
    let weak = ui.as_weak();
    ui.on_choose_path(move |kind| {
        if let Some(ui) = weak.upgrade() {
            let dialog = rfd::FileDialog::new();
            let path = match kind {
                0 => dialog
                    .set_title("选择服务程序")
                    .add_filter("Windows 程序", &["exe"])
                    .pick_file(),
                1 => dialog.set_title("选择工作目录").pick_folder(),
                2 => dialog
                    .set_title("选择服务配置")
                    .add_filter("TOML 配置", &["toml"])
                    .pick_file(),
                _ => None,
            };
            if let Some(path) = path {
                let path = path.to_string_lossy().into_owned().into();
                match kind {
                    0 => ui.set_executable(path),
                    1 => ui.set_working_directory(path),
                    2 => ui.set_config_path(path),
                    _ => {}
                }
                ui.set_launcher_dirty(true);
            }
        }
    });
    let weak = ui.as_weak();
    ui.on_open_admin(move || {
        if let Some(ui) = weak.upgrade()
            && teacher_oa_desktop::settings::validate_base_url(ui.get_base_url().as_str()).is_ok()
            && webbrowser::open(&format!(
                "{}/login",
                ui.get_base_url().trim_end_matches('/')
            ))
            .is_err()
        {
            show_feedback(&ui, "无法打开浏览器", true);
        }
    });
    let weak = ui.as_weak();
    ui.on_minimize_window(move || {
        if let Some(ui) = weak.upgrade() {
            ui.window().set_minimized(true);
        }
    });
    let weak = ui.as_weak();
    ui.on_toggle_maximize(move || {
        if let Some(ui) = weak.upgrade() {
            ui.window().set_maximized(!ui.window().is_maximized());
        }
    });
    let weak = ui.as_weak();
    let sender = commands;
    ui.window().on_close_requested(move || {
        let Some(ui) = weak.upgrade() else {
            return slint::CloseRequestResponse::HideWindow;
        };
        if ui.get_busy() {
            show_feedback(&ui, "请等待当前操作完成", true);
            return slint::CloseRequestResponse::KeepWindowShown;
        }
        if (!ui.get_running() && !ui.get_launcher_dirty() && !ui.get_config_dirty())
            || confirm(4, ui.get_running())
        {
            send(&ui, &sender, Command::Exit);
        }
        slint::CloseRequestResponse::KeepWindowShown
    });
    let weak = ui.as_weak();
    let timer = slint::Timer::default();
    timer.start(
        slint::TimerMode::Repeated,
        Duration::from_millis(250),
        move || {
            let Some(ui) = weak.upgrade() else {
                return;
            };
            for event in responses.try_iter() {
                match event {
                    Event::View(view) => {
                        if ui.get_pid().as_str() != view.pid {
                            ui.set_config_loaded(false);
                        }
                        ui.set_status(view.status.into());
                        ui.set_status_color(slint::Color::from_rgb_u8(
                            view.color.0,
                            view.color.1,
                            view.color.2,
                        ));
                        ui.set_pid(view.pid.into());
                        ui.set_uptime(view.uptime.into());
                        ui.set_dependencies(view.dependencies.into());
                        ui.set_version(view.version.into());
                        ui.set_running(view.running);
                        ui.set_connected(view.connected);
                        ui.set_can_force(view.can_force);
                        ui.set_busy(false);
                        if !view.running {
                            ui.set_config_loaded(false);
                        }
                    }
                    Event::Logs(text) => *log_text.borrow_mut() = text,
                    Event::Config(config) => {
                        set_config(&ui, &config);
                        *snapshot.borrow_mut() = Some(config);
                    }
                    Event::Feedback(message, error) => show_feedback(&ui, &message, error),
                    Event::LauncherSaved(settings) => set_launcher(&ui, &settings),
                    Event::Quit => {
                        let _ = slint::quit_event_loop();
                    }
                }
            }
            if !ui.get_log_paused() {
                let filter = ui.get_log_filter().to_lowercase();
                let text = log_text.borrow();
                let filtered = if filter.is_empty() {
                    text.clone()
                } else {
                    text.lines()
                        .filter(|line| line.to_lowercase().contains(&filter))
                        .collect::<Vec<_>>()
                        .join("\n")
                };
                ui.set_logs(filtered.into());
            }
        },
    );
    // Keep the timer alive for the application lifetime without leaking it.
    TIMER.with(|stored| *stored.borrow_mut() = Some(timer));
    install_snapshot(ui);
}

thread_local! { static TIMER: RefCell<Option<slint::Timer>> = const { RefCell::new(None) }; }

fn install_snapshot(ui: &AppWindow) {
    let mut args = std::env::args().skip(1);
    if args.next().as_deref() != Some("--screenshot") {
        return;
    }
    let Some(path) = args.next() else {
        return;
    };
    if let Some(page) = args.next().and_then(|value| value.parse().ok()) {
        ui.set_page(page);
    }
    let weak = ui.as_weak();
    slint::Timer::single_shot(Duration::from_millis(800), move || {
        if let Some(ui) = weak.upgrade()
            && let Ok(buffer) = ui.window().take_snapshot()
        {
            let _ = image::save_buffer(
                &path,
                buffer.as_bytes(),
                buffer.width(),
                buffer.height(),
                image::ColorType::Rgba8,
            );
        }
        let _ = slint::quit_event_loop();
    });
}

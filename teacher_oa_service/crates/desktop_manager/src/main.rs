#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod interface;
use slint::ComponentHandle;

slint::include_modules!();

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ui = AppWindow::new()?;
    let settings = teacher_oa_desktop::settings::load();
    let initial = settings.as_ref().cloned().unwrap_or_default();
    interface::set_launcher(&ui, &initial);
    if let Err(error) = settings {
        interface::show_feedback(&ui, &error, true);
    }
    let (commands, receiver) = std::sync::mpsc::channel();
    let (events, responses) = std::sync::mpsc::sync_channel(16);
    let worker = std::thread::Builder::new()
        .name("service-supervisor".into())
        .spawn(move || teacher_oa_desktop::worker::run(initial, receiver, events))?;
    interface::bind(&ui, commands, responses);
    ui.run()?;
    // The UI requests graceful shutdown before it exits; join only that completed worker.
    if worker.is_finished() {
        let _ = worker.join();
    }
    Ok(())
}

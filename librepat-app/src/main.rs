#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod app;
mod appliance_detail;
mod appliance_table;
mod appliance_view;
mod import_flow;
mod importers;
mod job_view;
mod modals;
mod reports_view;
mod session;
mod shell;
mod task;
mod theme;
mod viewer;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Glow,
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([980.0, 640.0]),
        ..eframe::NativeOptions::default()
    };
    eframe::run_native(
        "LibrePAT",
        options,
        Box::new(|creation_context| Ok(Box::new(app::LibrePatApp::new(creation_context)))),
    )
}

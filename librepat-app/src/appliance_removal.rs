use eframe::egui;

use crate::session::{JobSession, TableState};

pub(crate) fn show(
    context: &egui::Context,
    session: &mut JobSession,
    table: &mut TableState,
) -> Option<String> {
    let request = table.removal_request.clone()?;
    let count = request.selection.len();
    let (title, action, message) = if request.removed {
        (
            "Remove appliances",
            "Remove",
            format!(
                "Remove {count} selected appliances from the active job and reports? Their imported records and readings will remain preserved."
            ),
        )
    } else {
        (
            "Restore appliances",
            "Restore",
            format!("Restore {count} selected appliances to the active job and reports?"),
        )
    };
    let mut open = true;
    let mut confirmed = false;
    let mut cancelled = false;
    egui::Window::new(title)
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(context, |ui| {
            ui.set_max_width(420.0);
            ui.label(message);
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                confirmed = ui.button(action).clicked();
                cancelled = ui.button("Cancel").clicked();
            });
        });
    if confirmed {
        if let Err(error) = session.set_removed(&request.selection, request.removed) {
            table.removal_request = None;
            return Some(error);
        }
        table.selection.clear();
        table.selection_anchor = None;
        table.detail = None;
        table.removal_request = None;
    } else if cancelled || !open {
        table.removal_request = None;
    }
    None
}

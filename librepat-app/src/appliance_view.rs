use eframe::egui::{self, RichText};
use librepat_core::{ApplianceStatus, BulkDateEdit, DateField};

use crate::{
    session::{JobSession, TableState, parse_date, visible_indices},
    theme,
};

pub(crate) fn show(
    context: &egui::Context,
    ui: &mut egui::Ui,
    session: &mut JobSession,
    table: &mut TableState,
) -> Option<String> {
    let mut message = None;
    ui.heading("Appliances");
    ui.label(
        RichText::new(appliance_summary(session))
            .size(15.0)
            .strong(),
    );
    ui.label(
        RichText::new("Select rows for date edits; double-click any row for full details.")
            .color(theme::MUTED),
    );
    ui.add_space(12.0);
    filter_controls(ui, session, table);
    if !table.selection.is_empty() {
        ui.add_space(8.0);
        if let Some(error) = selection_controls(ui, session, table) {
            message = Some(error);
        }
    }
    ui.add_space(8.0);
    ui.separator();
    crate::appliance_table::show(ui, session, table);
    if let Some(error) = crate::appliance_detail::show(context, session, table) {
        message = Some(error);
    }
    message
}

fn appliance_summary(session: &JobSession) -> String {
    let pass = session
        .job
        .appliances
        .iter()
        .filter(|appliance| appliance.status == ApplianceStatus::Pass)
        .count();
    let fail = session.job.appliances.len() - pass;
    format!(
        "{} appliances · {pass} pass · {fail} fail",
        session.job.appliances.len()
    )
}

fn filter_controls(ui: &mut egui::Ui, session: &JobSession, table: &mut TableState) {
    ui.horizontal_wrapped(|ui| {
        ui.label(
            RichText::new("FILTER")
                .size(10.0)
                .strong()
                .color(theme::TEAL),
        );
        let filter_width = (ui.available_width() * 0.55).clamp(220.0, 520.0);
        ui.add_sized(
            [filter_width, 26.0],
            egui::TextEdit::singleline(&mut table.filter)
                .hint_text("ID, description, location, date, status"),
        );
        if ui.button("Select visible").clicked() {
            table.selection.extend(visible_indices(&session.job, table));
        }
    });
}

fn selection_controls(
    ui: &mut egui::Ui,
    session: &mut JobSession,
    table: &mut TableState,
) -> Option<String> {
    let mut clear = false;
    let selection_count = table.selection.len();
    let appliance_label = if selection_count == 1 {
        "APPLIANCE"
    } else {
        "APPLIANCES"
    };
    let result = egui::Frame::new()
        .fill(theme::TEAL_WASH)
        .inner_margin(egui::Margin::symmetric(12, 10))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("{selection_count} {appliance_label} SELECTED"))
                        .size(11.0)
                        .strong()
                        .color(theme::TEAL_DARK),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    clear = ui.button("Clear selection").clicked();
                });
            });
            ui.add_space(4.0);
            bulk_controls(ui, session, table)
        })
        .inner;
    if clear {
        table.selection.clear();
    }
    result
}

fn bulk_controls(
    ui: &mut egui::Ui,
    session: &mut JobSession,
    table: &mut TableState,
) -> Option<String> {
    let mut request = None;
    ui.horizontal_wrapped(|ui| {
        ui.label(
            RichText::new("SET DATE")
                .size(10.0)
                .strong()
                .color(theme::TEAL),
        );
        ui.add(
            egui::TextEdit::singleline(&mut table.exact_date)
                .desired_width(112.0)
                .hint_text("YYYY-MM-DD"),
        );
        if ui.button("Set test").clicked() {
            request = Some(BulkRequest::Set(DateField::Test));
        }
        if ui.button("Set retest").clicked() {
            request = Some(BulkRequest::Set(DateField::Retest));
        }
        ui.separator();
        ui.label(
            RichText::new("SHIFT")
                .size(10.0)
                .strong()
                .color(theme::TEAL),
        );
        ui.add(
            egui::TextEdit::singleline(&mut table.shift_days)
                .desired_width(64.0)
                .hint_text("days"),
        );
        if ui.button("Shift test").clicked() {
            request = Some(BulkRequest::Shift(DateField::Test));
        }
        if ui.button("Shift retest").clicked() {
            request = Some(BulkRequest::Shift(DateField::Retest));
        }
    });
    let request = request?;
    let edit = match request {
        BulkRequest::Set(field) => BulkDateEdit::Set {
            field,
            value: match parse_date(&table.exact_date) {
                Ok(value) => value,
                Err(error) => return Some(error),
            },
        },
        BulkRequest::Shift(field) => {
            let Ok(days) = table.shift_days.trim().parse::<i64>() else {
                return Some("Enter a signed whole number of days".into());
            };
            BulkDateEdit::Shift { field, days }
        }
    };
    session
        .apply_bulk(&table.selection, edit)
        .err()
        .map(|error| error.to_string())
}

enum BulkRequest {
    Set(DateField),
    Shift(DateField),
}

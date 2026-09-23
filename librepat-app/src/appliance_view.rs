use eframe::egui::{self, Key, KeyboardShortcut, Modifiers, RichText};
use librepat_core::{ApplianceStatus, BulkDateEdit, DateField};

use crate::{
    session::{JobSession, TableState, parse_date, visible_indices},
    theme,
};

const SELECT_VISIBLE_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::A);

pub(crate) fn show(
    context: &egui::Context,
    ui: &mut egui::Ui,
    session: &mut JobSession,
    table: &mut TableState,
) -> Option<String> {
    let mut message = None;
    if select_visible_shortcut(context) {
        select_visible(session, table);
    }
    ui.heading(if table.show_removed {
        "Removed appliances"
    } else {
        "Appliances"
    });
    ui.label(
        RichText::new(appliance_summary(session, table.show_removed))
            .size(15.0)
            .strong(),
    );
    ui.label(
        RichText::new(if table.show_removed {
            "Select appliances to restore to the active job.".into()
        } else {
            format!(
                "Shift-click selects a range. {} selects visible rows. Double-click a row for details.",
                context.format_shortcut(&SELECT_VISIBLE_SHORTCUT)
            )
        })
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
    if let Some(error) = crate::appliance_removal::show(context, session, table) {
        message = Some(error);
    }
    message
}

fn appliance_summary(session: &JobSession, show_removed: bool) -> String {
    if show_removed {
        let removed = session
            .job
            .appliances
            .iter()
            .filter(|appliance| appliance.removed)
            .count();
        return format!("{removed} removed appliances");
    }
    let pass = session
        .job
        .appliances
        .iter()
        .filter(|appliance| !appliance.removed && appliance.status == ApplianceStatus::Pass)
        .count();
    let total = session
        .job
        .appliances
        .iter()
        .filter(|appliance| !appliance.removed)
        .count();
    let fail = total - pass;
    format!("{total} appliances · {pass} pass · {fail} fail")
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
        if ui
            .button("Select visible")
            .on_hover_text(format!(
                "{} selects all visible appliances",
                ui.ctx().format_shortcut(&SELECT_VISIBLE_SHORTCUT)
            ))
            .clicked()
        {
            select_visible(session, table);
        }
        let removed = session
            .job
            .appliances
            .iter()
            .filter(|appliance| appliance.removed)
            .count();
        let label = if table.show_removed {
            "Back to active".into()
        } else {
            format!("Removed ({removed})")
        };
        if ui
            .add_enabled(table.show_removed || removed > 0, egui::Button::new(label))
            .clicked()
        {
            table.show_removed = !table.show_removed;
            table.selection.clear();
            table.selection_anchor = None;
            table.detail = None;
        }
    });
}

fn select_visible_shortcut(context: &egui::Context) -> bool {
    !context.text_edit_focused()
        && context.input_mut(|input| input.consume_shortcut(&SELECT_VISIBLE_SHORTCUT))
}

fn select_visible(session: &JobSession, table: &mut TableState) {
    table.selection.extend(visible_indices(&session.job, table));
    table.selection_anchor = None;
}

fn selection_controls(
    ui: &mut egui::Ui,
    session: &mut JobSession,
    table: &mut TableState,
) -> Option<String> {
    let mut clear = false;
    let mut request_removal = false;
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
                    request_removal = ui
                        .button(if table.show_removed {
                            "Restore selected"
                        } else {
                            "Remove selected"
                        })
                        .clicked();
                });
            });
            if table.show_removed {
                None
            } else {
                ui.add_space(4.0);
                bulk_controls(ui, session, table)
            }
        })
        .inner;
    if clear {
        table.selection.clear();
        table.selection_anchor = None;
    }
    if request_removal {
        table.removal_request = Some(crate::session::RemovalRequest {
            selection: table.selection.clone(),
            removed: !table.show_removed,
        });
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

#[cfg(test)]
#[path = "appliance_view_tests.rs"]
mod tests;

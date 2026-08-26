use eframe::egui::{self, RichText};
use librepat_core::latest_test_indices;

use crate::{
    session::{JobSession, TableState, format_time, parse_date, status_text},
    theme,
};

pub(crate) fn show(
    context: &egui::Context,
    session: &mut JobSession,
    table: &mut TableState,
) -> Option<String> {
    let detail = table.detail.as_mut()?;
    if detail.index >= session.job.appliances.len() {
        table.detail = None;
        return Some("The selected appliance no longer exists".into());
    }
    let mut open = true;
    egui::Window::new("Appliance details")
        .open(&mut open)
        .default_width(690.0)
        .resizable(true)
        .show(context, |ui| {
            let appliance = &mut session.job.appliances[detail.index];
            ui.label(
                RichText::new(format!("SOURCE RECORD {}", appliance.source_record_number))
                    .size(10.0)
                    .strong()
                    .color(theme::TEAL),
            );
            let mut changed = false;
            egui::Grid::new("appliance_edit_fields")
                .num_columns(2)
                .show(ui, |ui| {
                    for (label, value) in [
                        ("Appliance ID", &mut appliance.appliance_id),
                        ("Description", &mut appliance.description),
                        ("Location", &mut appliance.location),
                    ] {
                        ui.label(label);
                        changed |= ui.text_edit_singleline(value).changed();
                        ui.end_row();
                    }
                    ui.label("Test date");
                    ui.text_edit_singleline(&mut detail.test_date);
                    ui.end_row();
                    ui.label("Retest date");
                    ui.text_edit_singleline(&mut detail.retest_date);
                    ui.end_row();
                    ui.label("Imported test time");
                    let test_time = format_time(appliance.test_time);
                    ui.label(if test_time.is_empty() { "—" } else { &test_time });
                    ui.end_row();
                    ui.label("Imported comments");
                    ui.label(if appliance.comments.is_empty() {
                        "—"
                    } else {
                        &appliance.comments
                    });
                    ui.end_row();
                });
            if ui.button("Apply dates").clicked() {
                match (
                    parse_date(&detail.test_date),
                    parse_date(&detail.retest_date),
                ) {
                    (Ok(test_date), Ok(retest_date)) => {
                        appliance.test_date = test_date;
                        appliance.retest_date = retest_date;
                        detail.error = None;
                        changed = true;
                    }
                    (Err(error), _) | (_, Err(error)) => detail.error = Some(error),
                }
            }
            if let Some(error) = &detail.error {
                ui.colored_label(theme::FAIL, error);
            }
            session.dirty |= changed;
            ui.add_space(12.0);
            ui.label(
                RichText::new(format!(
                    "IMMUTABLE ELECTRICAL RESULTS · {}",
                    status_text(appliance.status).to_ascii_uppercase()
                ))
                .size(11.0)
                .strong()
                .color(theme::status_color(appliance.status)),
            );
            ui.label(
                RichText::new(
                    "The final occurrence of each test determines the current result; earlier attempts remain preserved.",
                )
                .size(10.0)
                .color(theme::MUTED),
            );
            ui.separator();
            result_grid(ui, appliance);
        });
    if !open {
        table.detail = None;
    }
    None
}

fn result_grid(ui: &mut egui::Ui, appliance: &librepat_core::Appliance) {
    let current = latest_test_indices(&appliance.tests);
    egui::ScrollArea::vertical()
        .max_height(280.0)
        .show(ui, |ui| {
            egui::Grid::new("result_details")
                .num_columns(5)
                .striped(true)
                .show(ui, |ui| {
                    ui.strong("Test");
                    ui.strong("Reading");
                    ui.strong("Limit");
                    ui.strong("Outcome");
                    ui.strong("Recorded");
                    ui.end_row();
                    for (index, test) in appliance.tests.iter().enumerate() {
                        ui.label(&test.raw_name);
                        ui.label(
                            test.measurement
                                .as_ref()
                                .map_or_else(|| "—".into(), |value| value.display_value()),
                        );
                        ui.label(test.limit.as_ref().map_or_else(
                            || "—".into(),
                            |value| {
                                format!(
                                    "{}{} {}",
                                    value.comparison.symbol(),
                                    value.value,
                                    value.unit
                                )
                            },
                        ));
                        ui.label(format!("{:?}", test.status));
                        if current.contains(&index) {
                            ui.label(RichText::new("Current").strong().color(theme::TEAL));
                        } else {
                            ui.label(RichText::new("Earlier attempt").color(theme::MUTED));
                        }
                        ui.end_row();
                    }
                });
        });
}

use eframe::egui::{self, RichText};
use librepat_core::ReportDateFormat;
use librepat_report::summarize_job;

use crate::{session::JobSession, task::ReportKind, theme};

pub(crate) fn show(ui: &mut egui::Ui, session: &mut JobSession, busy: bool) -> Option<ReportKind> {
    let mut action = None;
    let summary = summarize_job(&session.job);
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.heading("Reports");
        ui.label(
            RichText::new("PDFs use current metadata and read-only imported results.")
                .color(theme::MUTED),
        );
        ui.add_space(20.0);
        ui.horizontal(|ui| {
            stat(ui, "APPLIANCES", summary.appliances, theme::TEAL);
            stat(ui, "PASS", summary.passed, theme::PASS);
            stat(ui, "FAIL", summary.failed, theme::FAIL);
        });
        ui.add_space(20.0);
        date_format_selector(ui, session);
        ui.add_space(28.0);
        report_cards(ui, busy, &mut action);
        ui.add_space(30.0);
        let validity = session.job.metadata.valid_until;
        ui.label(
            RichText::new("CERTIFICATE VALIDITY")
                .size(11.0)
                .strong()
                .color(theme::TEAL),
        );
        ui.label(validity.map_or_else(
            || "No validity date set".into(),
            |date| session.job.metadata.report_date_format.format(date),
        ));
        ui.add_space(16.0);
        ui.label(
            RichText::new("Use your system PDF viewer to print. LibrePAT does not print directly.")
                .size(12.0)
                .color(theme::MUTED),
        );
    });
    action
}

fn report_cards(ui: &mut egui::Ui, busy: bool, action: &mut Option<ReportKind>) {
    let columns = if ui.available_width() >= 700.0 { 2 } else { 1 };
    ui.columns(columns, |columns| {
        report_card(
            &mut columns[0],
            "Appliance results",
            "A4 portrait · appliances in ID order · inline test details",
            "Generate results PDF",
            busy,
            action,
            ReportKind::Appliance,
        );
        let certificate_column = columns.len() - 1;
        if certificate_column == 0 {
            columns[0].add_space(12.0);
        }
        report_card(
            &mut columns[certificate_column],
            "Completion certificate",
            "A4 portrait · totals, testing period, declaration",
            "Generate certificate PDF",
            busy,
            action,
            ReportKind::Certificate,
        );
    });
}

fn date_format_selector(ui: &mut egui::Ui, session: &mut JobSession) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("PDF DATE FORMAT")
                .size(11.0)
                .strong()
                .color(theme::TEAL),
        );
        let previous = session.job.metadata.report_date_format;
        egui::ComboBox::from_id_salt("report_date_format")
            .selected_text(previous.label())
            .show_ui(ui, |ui| {
                for format in ReportDateFormat::ALL {
                    ui.selectable_value(
                        &mut session.job.metadata.report_date_format,
                        format,
                        format.label(),
                    );
                }
            });
        session.dirty |= previous != session.job.metadata.report_date_format;
        ui.label(
            RichText::new("Saved with this job and used in both PDFs.")
                .size(11.0)
                .color(theme::MUTED),
        );
    });
}

fn stat(ui: &mut egui::Ui, label: &str, value: usize, color: egui::Color32) {
    egui::Frame::new()
        .fill(egui::Color32::from_rgb(232, 232, 223))
        .inner_margin(egui::Margin::symmetric(18, 12))
        .show(ui, |ui| {
            ui.label(
                RichText::new(value.to_string())
                    .size(22.0)
                    .strong()
                    .color(color),
            );
            ui.label(RichText::new(label).size(10.0).color(theme::MUTED));
        });
}

fn report_card(
    ui: &mut egui::Ui,
    title: &str,
    description: &str,
    button: &str,
    busy: bool,
    action: &mut Option<ReportKind>,
    kind: ReportKind,
) {
    egui::Frame::group(ui.style())
        .inner_margin(egui::Margin::same(18))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new(title).size(17.0).strong().color(theme::INK));
            ui.label(RichText::new(description).color(theme::MUTED));
            ui.add_space(18.0);
            if ui.add_enabled(!busy, egui::Button::new(button)).clicked() {
                *action = Some(kind);
            }
        });
}

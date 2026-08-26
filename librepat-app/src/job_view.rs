use eframe::egui::{self, RichText};
use librepat_core::{JobMetadata, PostalAddress};

use crate::{
    session::{JobSession, parse_date},
    theme,
};

const WIDE_LAYOUT_WIDTH: f32 = 900.0;
const LABEL_WIDTH: f32 = 128.0;

pub(crate) fn show(ui: &mut egui::Ui, session: &mut JobSession) -> Option<String> {
    let mut changed = false;
    let mut error = None;
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.heading("Job details");
        ui.label(
            RichText::new("Editable metadata used by the report and certificate.")
                .color(theme::MUTED),
        );
        ui.add_space(18.0);

        if ui.available_width() >= WIDE_LAYOUT_WIDTH {
            wide_layout(ui, session, &mut changed, &mut error);
        } else {
            narrow_layout(ui, session, &mut changed, &mut error);
        }
        ui.add_space(32.0);
    });
    session.dirty |= changed;
    error
}

fn wide_layout(
    ui: &mut egui::Ui,
    session: &mut JobSession,
    changed: &mut bool,
    error: &mut Option<String>,
) {
    ui.columns(2, |columns| {
        *changed |= customer_section(&mut columns[0], &mut session.job.metadata);
        *changed |= site_section(&mut columns[1], &mut session.job.metadata);
    });
    ui.add_space(18.0);
    ui.columns(2, |columns| {
        *changed |= contractor_section(&mut columns[0], &mut session.job.metadata);
        *changed |= certificate_section(
            &mut columns[1],
            &mut session.job.metadata,
            &mut session.valid_until_text,
            error,
        );
    });
}

fn narrow_layout(
    ui: &mut egui::Ui,
    session: &mut JobSession,
    changed: &mut bool,
    error: &mut Option<String>,
) {
    *changed |= customer_section(ui, &mut session.job.metadata);
    *changed |= site_section(ui, &mut session.job.metadata);
    *changed |= contractor_section(ui, &mut session.job.metadata);
    *changed |= certificate_section(
        ui,
        &mut session.job.metadata,
        &mut session.valid_until_text,
        error,
    );
}

fn customer_section(ui: &mut egui::Ui, metadata: &mut JobMetadata) -> bool {
    section(ui, "CUSTOMER");
    let mut changed = field(ui, "Customer name", &mut metadata.customer_name);
    changed |= address(ui, &mut metadata.customer_address);
    changed
}

fn site_section(ui: &mut egui::Ui, metadata: &mut JobMetadata) -> bool {
    section(ui, "SITE");
    let mut changed = address_copy_actions(ui, metadata);
    changed |= field(ui, "Site name", &mut metadata.site_name);
    changed |= address(ui, &mut metadata.site_address);
    changed
}

fn contractor_section(ui: &mut egui::Ui, metadata: &mut JobMetadata) -> bool {
    section(ui, "CONTRACTOR & TESTER");
    let mut changed = field(ui, "Contractor", &mut metadata.contractor_name);
    changed |= address(ui, &mut metadata.contractor_address);
    changed |= field(ui, "Telephone", &mut metadata.contractor_telephone);
    changed |= field(ui, "Email", &mut metadata.contractor_email);
    changed |= field(ui, "Tester", &mut metadata.tester_name);
    changed
}

fn certificate_section(
    ui: &mut egui::Ui,
    metadata: &mut JobMetadata,
    valid_until_text: &mut String,
    error: &mut Option<String>,
) -> bool {
    section(ui, "CERTIFICATE");
    let mut changed = field(ui, "Certificate number", &mut metadata.certificate_number);
    changed |= field(ui, "Order reference", &mut metadata.order_reference);
    changed |= date_field(
        ui,
        "Valid until",
        valid_until_text,
        &mut metadata.valid_until,
        error,
    );
    ui.add_space(8.0);
    ui.label(RichText::new("Report notes").strong());
    changed |= ui
        .add(
            egui::TextEdit::multiline(&mut metadata.report_notes)
                .desired_rows(5)
                .desired_width(f32::INFINITY),
        )
        .changed();
    changed
}

fn address_copy_actions(ui: &mut egui::Ui, metadata: &mut JobMetadata) -> bool {
    let mut changed = false;
    let customer = metadata.customer_address.clone();
    let site = metadata.site_address.clone();
    ui.horizontal_wrapped(|ui| {
        if ui
            .add_enabled(
                customer != PostalAddress::default() && customer != site,
                egui::Button::new("Use customer address"),
            )
            .clicked()
        {
            metadata.site_address.clone_from(&customer);
            changed = true;
        }
        if ui
            .add_enabled(
                site != PostalAddress::default() && site != customer,
                egui::Button::new("Copy site to customer"),
            )
            .clicked()
        {
            metadata.customer_address.clone_from(&site);
            changed = true;
        }
    });
    changed
}

fn section(ui: &mut egui::Ui, title: &str) {
    ui.add_space(12.0);
    ui.label(RichText::new(title).size(12.0).strong().color(theme::TEAL));
    ui.separator();
    ui.add_space(2.0);
}

fn field(ui: &mut egui::Ui, label: &str, value: &mut String) -> bool {
    ui.horizontal(|ui| {
        ui.add_sized(
            [LABEL_WIDTH, 22.0],
            egui::Label::new(RichText::new(label).strong()),
        );
        ui.add_sized(
            [ui.available_width().max(160.0), 24.0],
            egui::TextEdit::singleline(value),
        )
        .changed()
    })
    .inner
}

fn address(ui: &mut egui::Ui, value: &mut PostalAddress) -> bool {
    let mut changed = false;
    for (label, field) in [
        ("Address line 1", &mut value.line_1),
        ("Address line 2", &mut value.line_2),
        ("Town / city", &mut value.town),
        ("County", &mut value.county),
        ("Postcode", &mut value.postcode),
    ] {
        changed |= self::field(ui, label, field);
    }
    changed
}

fn date_field(
    ui: &mut egui::Ui,
    label: &str,
    text: &mut String,
    value: &mut Option<time::Date>,
    error: &mut Option<String>,
) -> bool {
    let response = ui
        .horizontal(|ui| {
            ui.add_sized(
                [LABEL_WIDTH, 22.0],
                egui::Label::new(RichText::new(label).strong()),
            );
            let response = ui.add_sized([150.0, 24.0], egui::TextEdit::singleline(text));
            ui.label(RichText::new("YYYY-MM-DD").size(11.0).color(theme::MUTED));
            response
        })
        .inner;
    if !response.lost_focus() {
        return false;
    }
    match parse_date(text) {
        Ok(parsed) if *value != parsed => {
            *value = parsed;
            true
        }
        Ok(_) => false,
        Err(message) => {
            *error = Some(message);
            false
        }
    }
}

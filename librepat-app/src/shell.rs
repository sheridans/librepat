use eframe::egui::{self, Color32, RichText};

use crate::{session::JobSession, theme};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum View {
    Job,
    Appliances,
    Reports,
}

#[derive(Clone, Copy)]
pub(crate) enum ShellAction {
    Import,
    Open,
    Save,
}

pub(crate) fn header(
    root: &mut egui::Ui,
    session: Option<&JobSession>,
    busy: bool,
) -> Option<ShellAction> {
    let mut action = None;
    egui::Panel::top("header")
        .frame(
            egui::Frame::new()
                .fill(theme::INK)
                .inner_margin(egui::Margin::symmetric(18, 12)),
        )
        .show(root, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("LIBREPAT")
                        .size(19.0)
                        .strong()
                        .color(Color32::WHITE),
                );
                ui.label(
                    RichText::new("FIELD RECORD")
                        .size(10.0)
                        .color(Color32::from_gray(170)),
                );
                ui.separator();
                if let Some(session) = session {
                    let path = session.store.path();
                    let file_name = path
                        .file_name()
                        .unwrap_or(path.as_os_str())
                        .to_string_lossy();
                    ui.label(
                        RichText::new(file_name)
                            .size(12.0)
                            .strong()
                            .color(Color32::from_gray(220)),
                    )
                    .on_hover_text(path.display().to_string());
                    let marker = if session.dirty { "UNSAVED" } else { "SAVED" };
                    ui.label(RichText::new(marker).size(10.0).color(if session.dirty {
                        Color32::from_rgb(245, 190, 75)
                    } else {
                        Color32::from_rgb(105, 203, 164)
                    }));
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_enabled_ui(!busy, |ui| {
                        if session.is_some()
                            && ui
                                .add(
                                    egui::Button::new(
                                        RichText::new("Save").strong().color(Color32::WHITE),
                                    )
                                    .fill(theme::TEAL),
                                )
                                .clicked()
                        {
                            action = Some(ShellAction::Save);
                        }
                        if ui.button("Open job").clicked() {
                            action = Some(ShellAction::Open);
                        }
                        if ui.button("Import tester export").clicked() {
                            action = Some(ShellAction::Import);
                        }
                    });
                });
            });
        });
    action
}

pub(crate) fn navigation(root: &mut egui::Ui, view: &mut View, session: &JobSession) {
    egui::Panel::left("navigation")
        .exact_size(194.0)
        .frame(
            egui::Frame::new()
                .fill(theme::TEAL_DARK)
                .inner_margin(egui::Margin::same(14)),
        )
        .show(root, |ui| {
            ui.add_space(12.0);
            ui.label(
                RichText::new(if session.job.metadata.site_name.is_empty() {
                    "UNTITLED SITE"
                } else {
                    &session.job.metadata.site_name
                })
                .size(15.0)
                .strong()
                .color(Color32::WHITE),
            );
            ui.label(
                RichText::new(format!("{} appliances", session.job.appliances.len()))
                    .size(11.0)
                    .color(Color32::from_gray(185)),
            );
            ui.add_space(24.0);
            nav_button(ui, view, View::Job, "01  JOB DETAILS");
            nav_button(ui, view, View::Appliances, "02  APPLIANCES");
            nav_button(ui, view, View::Reports, "03  REPORTS");
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.hyperlink_to(
                    RichText::new("SUPPORT LIBREPAT")
                        .size(10.0)
                        .color(Color32::from_gray(190)),
                    "https://buymeacoffee.com/sheridans",
                )
                .on_hover_text("Buy me a coffee");
                ui.add_space(5.0);
                ui.label(
                    RichText::new("Results are read-only")
                        .size(10.0)
                        .color(Color32::from_gray(165)),
                );
            });
        });
}

pub(crate) fn footer(root: &mut egui::Ui) {
    let year = time::OffsetDateTime::now_utc().year();
    egui::Panel::bottom("footer")
        .exact_size(30.0)
        .frame(
            egui::Frame::new()
                .fill(theme::INK)
                .inner_margin(egui::Margin::symmetric(18, 7)),
        )
        .show(root, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("© {} Sheridan Computers", copyright_years(year)))
                        .size(10.0)
                        .color(Color32::from_gray(180)),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.hyperlink_to("sheridancomputers.com", "https://sheridancomputers.com");
                });
            });
        });
}

fn copyright_years(current_year: i32) -> String {
    const FIRST_YEAR: i32 = 2026;
    if current_year <= FIRST_YEAR {
        FIRST_YEAR.to_string()
    } else {
        format!("{FIRST_YEAR}–{current_year}")
    }
}

pub(crate) fn start(root: &mut egui::Ui, busy: bool) -> Option<ShellAction> {
    let mut action = None;
    egui::CentralPanel::default().show(root, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(90.0);
            ui.label(
                RichText::new("Create or open a PAT job")
                    .heading()
                    .color(theme::INK),
            );
            ui.add_space(8.0);
            ui.label(
                RichText::new("Import a tester export, edit job metadata, and create PDF reports.")
                    .size(15.0)
                    .color(theme::MUTED),
            );
            ui.add_space(36.0);
            ui.add_enabled_ui(!busy, |ui| {
                if ui
                    .add_sized([220.0, 44.0], egui::Button::new("Import a tester export"))
                    .clicked()
                {
                    action = Some(ShellAction::Import);
                }
                ui.add_space(10.0);
                if ui
                    .add_sized([220.0, 44.0], egui::Button::new("Open a .librepat job"))
                    .clicked()
                {
                    action = Some(ShellAction::Open);
                }
            });
            ui.add_space(50.0);
            ui.label(
                RichText::new("ONE JOB · ONE FILE · ORIGINAL SOURCE EMBEDDED")
                    .size(10.0)
                    .color(theme::TEAL),
            );
        });
    });
    action
}

fn nav_button(ui: &mut egui::Ui, current: &mut View, target: View, label: &str) {
    let selected = *current == target;
    if ui
        .add_sized(
            [166.0, 38.0],
            egui::Button::selectable(selected, RichText::new(label).color(Color32::WHITE)),
        )
        .clicked()
    {
        *current = target;
    }
}

#[cfg(test)]
mod tests {
    use super::copyright_years;

    #[test]
    fn copyright_should_not_repeat_the_first_year() {
        assert_eq!(copyright_years(2026), "2026");
    }

    #[test]
    fn copyright_should_extend_to_a_later_year() {
        assert_eq!(copyright_years(2027), "2026–2027");
    }
}

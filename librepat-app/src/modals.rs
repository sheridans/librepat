use eframe::egui;

use crate::{
    app::{DeferredAction, LibrePatApp},
    viewer,
};

pub(crate) fn unsaved(app: &mut LibrePatApp, context: &egui::Context) {
    let Some(action) = app.pending else {
        return;
    };
    let mut decision = None;
    egui::Window::new("Unsaved changes")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(context, |ui| {
            ui.label("Save current metadata changes before continuing?");
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui.button("Save and continue").clicked() {
                    decision = Some(UnsavedDecision::Save);
                }
                if ui.button("Discard changes").clicked() {
                    decision = Some(UnsavedDecision::Discard);
                }
                if ui.button("Cancel").clicked() {
                    decision = Some(UnsavedDecision::Cancel);
                }
            });
        });
    match decision {
        Some(UnsavedDecision::Save) => {
            app.save();
            if app.session.as_ref().is_none_or(|session| !session.dirty) {
                app.pending = None;
                app.execute(action, context);
            }
        }
        Some(UnsavedDecision::Discard) => {
            app.pending = None;
            if matches!(action, DeferredAction::Close) {
                if let Some(session) = app.session.as_mut() {
                    session.dirty = false;
                }
                app.execute(action, context);
            } else if app.reload_current() {
                app.execute(action, context);
            }
        }
        Some(UnsavedDecision::Cancel) => app.pending = None,
        None => {}
    }
}

pub(crate) fn warnings(app: &mut LibrePatApp, context: &egui::Context) {
    let Some(review) = app.warning_review.as_ref() else {
        return;
    };
    let mut accepted = None;
    egui::Window::new("Import warnings")
        .collapsible(false)
        .default_width(620.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(context, |ui| {
            ui.label(format!(
                "{} recoverable issue(s) were found. Review them before accepting this job.",
                review.warnings.len()
            ));
            egui::ScrollArea::vertical()
                .max_height(280.0)
                .show(ui, |ui| {
                    for warning in &review.warnings {
                        let location = warning
                            .record
                            .as_ref()
                            .map_or_else(String::new, |record| format!("Record {record}: "));
                        ui.label(format!("{location}{}", warning.message));
                    }
                });
            ui.horizontal(|ui| {
                if ui.button("Accept and create job").clicked() {
                    accepted = Some(true);
                }
                if ui.button("Cancel import").clicked() {
                    accepted = Some(false);
                }
            });
        });
    if let Some(accepted) = accepted {
        let review = app.warning_review.take();
        if accepted && let Some(review) = review {
            app.create_imported_job(review.job, review.destination);
        }
    }
}

pub(crate) fn replacement(app: &mut LibrePatApp, context: &egui::Context) {
    let Some(review) = app.replacement_review.as_ref() else {
        return;
    };
    let mut decision = None;
    egui::Window::new("Replace existing job?")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(context, |ui| {
            ui.label("A LibrePAT job already exists at:");
            ui.label(review.destination.display().to_string());
            ui.label("The original is kept unless its replacement is created successfully.");
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui.button("Replace existing job").clicked() {
                    decision = Some(ReplacementDecision::Replace);
                }
                if ui.button("Choose another location").clicked() {
                    decision = Some(ReplacementDecision::ChooseAnother);
                }
                if ui.button("Cancel import").clicked() {
                    decision = Some(ReplacementDecision::Cancel);
                }
            });
        });
    if let Some(decision) = decision {
        let review = app.replacement_review.take();
        if let Some(review) = review {
            match decision {
                ReplacementDecision::Replace => app.tasks.import(
                    review.source,
                    crate::task::JobDestination::new(review.destination, true),
                ),
                ReplacementDecision::ChooseAnother => {
                    app.choose_import_destination(review.source);
                }
                ReplacementDecision::Cancel => {}
            }
        }
    }
}

pub(crate) fn messages(app: &mut LibrePatApp, context: &egui::Context) {
    if let Some((title, body)) = app.message.clone() {
        let mut close = false;
        egui::Window::new(title)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(context, |ui| {
                ui.label(body);
                if ui.button("Close").clicked() {
                    close = true;
                }
            });
        if close {
            app.message = None;
        }
    }
    pdf_ready(app, context);
}

fn pdf_ready(app: &mut LibrePatApp, context: &egui::Context) {
    let Some(path) = app.pdf_ready.clone() else {
        return;
    };
    let mut close = false;
    egui::Window::new("PDF created")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(context, |ui| {
            ui.label(path.display().to_string());
            ui.horizontal(|ui| {
                if ui.button("Open in PDF viewer").clicked() {
                    if let Err(error) = viewer::open(&path) {
                        app.message = Some(("Could not open PDF".into(), error.to_string()));
                    }
                    close = true;
                }
                if ui.button("Done").clicked() {
                    close = true;
                }
            });
        });
    if close {
        app.pdf_ready = None;
    }
}

enum UnsavedDecision {
    Save,
    Discard,
    Cancel,
}

enum ReplacementDecision {
    Replace,
    ChooseAnother,
    Cancel,
}

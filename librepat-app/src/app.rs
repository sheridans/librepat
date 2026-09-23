use std::path::{Path, PathBuf};

use eframe::egui::{self, RichText};

use crate::{
    appliance_view,
    import_flow::{ReplacementReview, WarningReview},
    job_view, reports_view,
    session::{JobSession, TableState},
    shell::{self, ShellAction, View},
    task::{ReportKind, TaskHub, WorkerResult},
    theme,
};

pub(crate) struct LibrePatApp {
    pub(crate) session: Option<JobSession>,
    view: View,
    table: TableState,
    pub(crate) tasks: TaskHub,
    pub(crate) pending: Option<DeferredAction>,
    pub(crate) warning_review: Option<WarningReview>,
    pub(crate) replacement_review: Option<ReplacementReview>,
    pub(crate) message: Option<(String, String)>,
    pub(crate) pdf_ready: Option<PathBuf>,
}

#[derive(Clone, Copy)]
pub(crate) enum DeferredAction {
    Import,
    Open,
    Close,
}

impl LibrePatApp {
    pub fn new(creation_context: &eframe::CreationContext<'_>) -> Self {
        theme::install(&creation_context.egui_ctx);
        Self {
            session: None,
            view: View::Job,
            table: TableState::default(),
            tasks: TaskHub::new(crate::importers::registry()),
            pending: None,
            warning_review: None,
            replacement_review: None,
            message: None,
            pdf_ready: None,
        }
    }

    fn handle_shell_action(&mut self, action: ShellAction, context: &egui::Context) {
        match action {
            ShellAction::Import => self.request(DeferredAction::Import, context),
            ShellAction::Open => self.request(DeferredAction::Open, context),
            ShellAction::Quit => self.request(DeferredAction::Close, context),
            ShellAction::Save => self.save(),
        }
    }

    fn request(&mut self, action: DeferredAction, context: &egui::Context) {
        if self.session.as_ref().is_some_and(|session| session.dirty) {
            self.pending = Some(action);
        } else {
            self.execute(action, context);
        }
    }

    pub(crate) fn execute(&mut self, action: DeferredAction, context: &egui::Context) {
        match action {
            DeferredAction::Import => self.choose_import(),
            DeferredAction::Open => self.choose_job(),
            DeferredAction::Close => context.send_viewport_cmd(egui::ViewportCommand::Close),
        }
    }

    fn choose_job(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("LibrePAT job", &["librepat"])
            .pick_file()
        else {
            return;
        };
        self.open_job(&path);
    }

    pub(crate) fn open_job(&mut self, path: &Path) {
        match JobSession::open(path) {
            Ok(session) => {
                self.session = Some(session);
                self.table = TableState::default();
                self.view = View::Job;
            }
            Err(error) => self.show_error("Could not open job", error.to_string()),
        }
    }

    pub(crate) fn save(&mut self) {
        let Some(session) = self.session.as_mut() else {
            return;
        };
        if let Err(error) = session.save() {
            self.show_error("Could not save job", error.to_string());
        }
    }

    fn start_report(&mut self, kind: ReportKind) {
        let Some(session) = self.session.as_ref() else {
            return;
        };
        let suggested = match kind {
            ReportKind::Appliance => "appliance-results.pdf",
            ReportKind::Certificate => "librepat-certificate-report.pdf",
        };
        let Some(destination) = rfd::FileDialog::new()
            .add_filter("PDF document", &["pdf"])
            .set_file_name(suggested)
            .save_file()
        else {
            return;
        };
        self.tasks.report(
            session.job.clone(),
            with_extension(destination, "pdf"),
            kind,
        );
    }

    fn receive_worker(&mut self) {
        let Some(result) = self.tasks.receive() else {
            return;
        };
        match result {
            WorkerResult::Imported {
                job,
                warnings,
                destination,
            } => {
                let job = *job;
                if warnings.is_empty() {
                    self.create_imported_job(job, destination);
                } else {
                    self.warning_review = Some(WarningReview {
                        job,
                        warnings,
                        destination,
                    });
                }
            }
            WorkerResult::Created(path) => self.open_job(&path),
            WorkerResult::Reported(path) => self.pdf_ready = Some(path),
            WorkerResult::Failed(error) => self.show_error("Task failed", error),
        }
    }

    pub(crate) fn reload_current(&mut self) -> bool {
        let Some(path) = self
            .session
            .as_ref()
            .map(|session| session.store.path().to_owned())
        else {
            return true;
        };
        match JobSession::open(&path) {
            Ok(session) => {
                self.session = Some(session);
                true
            }
            Err(error) => {
                self.show_error("Could not discard changes", error.to_string());
                false
            }
        }
    }

    pub(crate) fn show_error(&mut self, title: &str, body: String) {
        self.message = Some((title.into(), body));
    }
}

impl eframe::App for LibrePatApp {
    fn ui(&mut self, root: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let context = root.ctx().clone();
        if context.input(|input| input.key_pressed(egui::Key::Tab)) {
            context.request_repaint();
        }
        self.receive_worker();
        if context.input(|input| input.viewport().close_requested())
            && self.session.as_ref().is_some_and(|session| session.dirty)
        {
            context.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.pending.get_or_insert(DeferredAction::Close);
        }

        let busy = self.tasks.is_busy();
        let drop_hovered = self.session.is_none()
            && !busy
            && context.input(|input| !input.raw.hovered_files.is_empty());
        let shell_action = shell::header(root, self.session.as_ref(), busy);
        shell::footer(root);
        let mut report_action = None;
        let mut view_message = None;
        if let Some(session) = self.session.as_mut() {
            shell::navigation(root, &mut self.view, session);
            egui::CentralPanel::default().show(root, |ui| match self.view {
                View::Job => view_message = job_view::show(ui, session),
                View::Appliances => {
                    view_message = appliance_view::show(&context, ui, session, &mut self.table);
                }
                View::Reports => report_action = reports_view::show(ui, session, busy),
            });
        } else if let Some(action) = shell::start(root, busy, drop_hovered) {
            self.handle_shell_action(action, &context);
        }
        if let Some(action) = shell_action {
            self.handle_shell_action(action, &context);
        }
        if let Some(kind) = report_action {
            self.start_report(kind);
        }
        if let Some(message) = view_message {
            self.show_error("Invalid edit", message);
        }
        self.handle_start_page_drop(&context);

        crate::modals::unsaved(self, &context);
        crate::modals::replacement(self, &context);
        crate::modals::warnings(self, &context);
        crate::modals::messages(self, &context);
        if let Some(label) = self.tasks.label() {
            egui::Window::new("Working")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(&context, |ui| {
                    ui.spinner();
                    ui.label(RichText::new(label).strong());
                });
            context.request_repaint_after(std::time::Duration::from_millis(100));
        }
    }
}

pub(crate) fn with_extension(mut path: PathBuf, extension: &str) -> PathBuf {
    let matches = path
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case(extension));
    if !matches {
        path.set_extension(extension);
    }
    path
}

#[cfg(test)]
mod tests {
    use super::with_extension;
    use std::path::PathBuf;

    #[test]
    fn with_extension_should_replace_a_different_extension() {
        assert_eq!(
            with_extension(PathBuf::from("job.sqlite"), "librepat"),
            PathBuf::from("job.librepat")
        );
    }

    #[test]
    fn with_extension_should_accept_case_insensitive_match() {
        assert_eq!(
            with_extension(PathBuf::from("report.PDF"), "pdf"),
            PathBuf::from("report.PDF")
        );
    }
}

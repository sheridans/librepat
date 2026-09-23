use std::path::Path;

use eframe::egui;
use librepat_core::ImporterMetadata;

use crate::app::LibrePatApp;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DropKind {
    Job,
    TesterExport,
}

impl LibrePatApp {
    pub(crate) fn handle_start_page_drop(&mut self, context: &egui::Context) {
        if self.session.is_some() || self.tasks.is_busy() {
            return;
        }
        let paths = context.input(|input| {
            input
                .raw
                .dropped_files
                .iter()
                .map(|file| file.path().to_owned())
                .collect::<Vec<_>>()
        });
        let [path] = paths.as_slice() else {
            if !paths.is_empty() {
                self.show_error("Could not open files", "Drop one file at a time".into());
            }
            return;
        };

        let metadata = self.tasks.importer_metadata();
        match drop_kind(path, &metadata) {
            Some(DropKind::Job) => self.open_job(path),
            Some(DropKind::TesterExport) => self.choose_import_destination(path.clone()),
            None => self.show_error(
                "Unsupported file",
                "Drop a .librepat job or a supported tester export".into(),
            ),
        }
    }
}

fn drop_kind(path: &Path, importer_metadata: &[ImporterMetadata]) -> Option<DropKind> {
    let extension = path.extension()?.to_str()?;
    if extension.eq_ignore_ascii_case("librepat") {
        return Some(DropKind::Job);
    }
    importer_metadata
        .iter()
        .any(|metadata| {
            metadata
                .extensions
                .iter()
                .any(|supported| extension.eq_ignore_ascii_case(supported))
        })
        .then_some(DropKind::TesterExport)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_IMPORTER: ImporterMetadata =
        ImporterMetadata::new("test", "test-format", "Test format", &["flk", "txt"]);

    #[test]
    fn drop_kind_should_open_job_extension_case_insensitively() {
        assert_eq!(
            drop_kind(Path::new("job.LIBREPAT"), &[]),
            Some(DropKind::Job)
        );
    }

    #[test]
    fn drop_kind_should_import_registered_extension_case_insensitively() {
        assert_eq!(
            drop_kind(Path::new("export.FLK"), &[TEST_IMPORTER]),
            Some(DropKind::TesterExport)
        );
    }

    #[test]
    fn drop_kind_should_reject_an_unsupported_extension() {
        assert_eq!(drop_kind(Path::new("report.pdf"), &[TEST_IMPORTER]), None);
    }
}

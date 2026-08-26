use std::{fs, path::PathBuf};

use librepat_core::{ImportWarning, Job};

use crate::{
    app::{LibrePatApp, with_extension},
    task::JobDestination,
};

pub(crate) struct WarningReview {
    pub(crate) job: Job,
    pub(crate) warnings: Vec<ImportWarning>,
    pub(crate) destination: JobDestination,
}

pub(crate) struct ReplacementReview {
    pub(crate) source: PathBuf,
    pub(crate) destination: PathBuf,
}

impl LibrePatApp {
    pub(crate) fn choose_import(&mut self) {
        let metadata = self.tasks.importer_metadata();
        let supported_extensions = unique_extensions(
            metadata
                .iter()
                .flat_map(|importer| importer.extensions.iter().copied()),
        );
        let supported_label = filter_label("Supported tester exports", &supported_extensions);
        let supported_variants = extension_variants(supported_extensions.iter().copied());
        let mut dialog = rfd::FileDialog::new().add_filter(&supported_label, &supported_variants);
        for importer in metadata {
            let label = filter_label(importer.display_name, importer.extensions);
            let variants = extension_variants(importer.extensions.iter().copied());
            dialog = dialog.add_filter(&label, &variants);
        }
        dialog = dialog.add_filter("All files", &["*"]);
        let Some(source) = dialog.pick_file() else {
            return;
        };
        self.choose_import_destination(source);
    }

    pub(crate) fn choose_import_destination(&mut self, source: PathBuf) {
        let Some(destination) = rfd::FileDialog::new()
            .add_filter("LibrePAT job", &["librepat"])
            .set_file_name("job.librepat")
            .save_file()
        else {
            return;
        };
        let destination = with_extension(destination, "librepat");
        if destination.exists() {
            self.replacement_review = Some(ReplacementReview {
                source,
                destination,
            });
        } else {
            self.tasks
                .import(source, JobDestination::new(destination, false));
        }
    }

    pub(crate) fn create_imported_job(&mut self, job: Job, destination: JobDestination) {
        let closes_current = destination.replace_existing
            && self
                .session
                .as_ref()
                .is_some_and(|session| same_file(session.store.path(), destination.path.as_path()));
        if closes_current {
            self.session = None;
        }
        self.tasks.create(job, destination);
    }
}

fn unique_extensions<'a>(extensions: impl IntoIterator<Item = &'a str>) -> Vec<&'a str> {
    let mut unique = Vec::new();
    for extension in extensions {
        if !unique.contains(&extension) {
            unique.push(extension);
        }
    }
    unique
}

fn filter_label(name: &str, extensions: &[&str]) -> String {
    let patterns = extensions
        .iter()
        .map(|extension| format!("*.{extension}"))
        .collect::<Vec<_>>()
        .join(", ");
    format!("{name} ({patterns})")
}

fn extension_variants<'a>(extensions: impl IntoIterator<Item = &'a str>) -> Vec<String> {
    let mut variants = Vec::new();
    for extension in extensions {
        for candidate in case_variants(extension) {
            if !variants.contains(&candidate) {
                variants.push(candidate);
            }
        }
    }
    variants
}

fn case_variants(extension: &str) -> Vec<String> {
    let mut variants = vec![String::new()];
    for character in extension.chars() {
        if character.is_ascii_alphabetic() {
            let prefixes = std::mem::take(&mut variants);
            for prefix in prefixes {
                let mut lowercase = prefix.clone();
                lowercase.push(character.to_ascii_lowercase());
                variants.push(lowercase);
                let mut uppercase = prefix;
                uppercase.push(character.to_ascii_uppercase());
                variants.push(uppercase);
            }
        } else {
            for variant in &mut variants {
                variant.push(character);
            }
        }
    }
    variants
}

fn same_file(left: &std::path::Path, right: &std::path::Path) -> bool {
    match (fs::canonicalize(left), fs::canonicalize(right)) {
        (Ok(left), Ok(right)) => left == right,
        _ => left == right,
    }
}

#[cfg(test)]
mod tests {
    use super::{extension_variants, filter_label, unique_extensions};

    #[test]
    fn import_filter_labels_show_the_supported_extensions() {
        assert_eq!(
            filter_label("Supported tester exports", &["flk", "txt", "asc"]),
            "Supported tester exports (*.flk, *.txt, *.asc)"
        );
        assert_eq!(
            filter_label("Fluke FLK text export", &["flk"]),
            "Fluke FLK text export (*.flk)"
        );
    }

    #[test]
    fn combined_filter_extensions_are_unique_and_case_insensitive() {
        let unique = unique_extensions(["flk", "txt", "asc", "txt", "asc"]);
        assert_eq!(unique, ["flk", "txt", "asc"]);
        assert_eq!(
            extension_variants(unique),
            [
                "flk", "flK", "fLk", "fLK", "Flk", "FlK", "FLk", "FLK", "txt", "txT", "tXt", "tXT",
                "Txt", "TxT", "TXt", "TXT", "asc", "asC", "aSc", "aSC", "Asc", "AsC", "ASc", "ASC",
            ]
        );
    }
}

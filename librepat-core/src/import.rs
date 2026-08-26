use std::path::Path;

use thiserror::Error;

use crate::{ImportProvenance, ImportedJob};

/// Borrowed source bytes and their display filename.
#[derive(Clone, Copy, Debug)]
pub struct ImportSource<'a> {
    pub filename: &'a str,
    pub bytes: &'a [u8],
}

impl<'a> ImportSource<'a> {
    #[must_use]
    pub fn new(filename: &'a str, bytes: &'a [u8]) -> Self {
        Self { filename, bytes }
    }

    #[must_use]
    pub fn extension(&self) -> Option<&str> {
        Path::new(self.filename).extension()?.to_str()
    }
}

/// Confidence that an importer understands a source.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Detection {
    NotRecognized,
    Possible,
    Certain,
}

/// Stable identity and user-facing details for one importer and source format.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ImporterMetadata {
    pub importer_id: &'static str,
    pub format_id: &'static str,
    pub display_name: &'static str,
    /// Lowercase filename extensions without a leading dot.
    pub extensions: &'static [&'static str],
}

impl ImporterMetadata {
    #[must_use]
    pub const fn new(
        importer_id: &'static str,
        format_id: &'static str,
        display_name: &'static str,
        extensions: &'static [&'static str],
    ) -> Self {
        Self {
            importer_id,
            format_id,
            display_name,
            extensions,
        }
    }

    #[must_use]
    pub fn provenance(self) -> ImportProvenance {
        ImportProvenance::new(self.importer_id, self.format_id)
    }
}

/// Category for a recoverable import warning.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ImportWarningKind {
    UnknownLine,
    InconsistentField,
    DuplicateId,
    IncompleteRecord,
    InvalidDate,
    InvalidTime,
    UnsupportedMode,
}

/// A recoverable issue associated with an optional record and source line.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportWarning {
    pub kind: ImportWarningKind,
    pub message: String,
    pub record: Option<String>,
    pub line: Option<usize>,
}

/// Fatal importer error used only when a source is unusable.
#[derive(Debug, Error)]
pub enum ImportError {
    #[error("the source is not a recognized tester export")]
    NotRecognized,
    #[error("the source text is not valid UTF-8")]
    InvalidText,
    #[error("the source contains non-ASCII text")]
    NonAscii,
    #[error("no usable appliance records were found")]
    NoUsableRecords,
    #[error("no importer recognized the source")]
    NoImporter,
    #[error("multiple importers matched the source equally: {format_ids}")]
    AmbiguousImporter { format_ids: String },
    #[error("importer `{format_id}` failed: {source}")]
    Importer {
        format_id: &'static str,
        #[source]
        source: Box<ImportError>,
    },
}

/// Synchronous boundary implemented by tester export importers.
pub trait TesterImporter: Send + Sync {
    fn metadata(&self) -> ImporterMetadata;
    fn detect(&self, source: &ImportSource<'_>) -> Detection;
    fn import(&self, source: &ImportSource<'_>) -> Result<ImportedJob, ImportError>;
}

/// Runtime registry for heterogeneous tester importers.
#[derive(Default)]
pub struct ImporterRegistry {
    importers: Vec<Box<dyn TesterImporter>>,
}

impl ImporterRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, importer: impl TesterImporter + 'static) {
        self.importers.push(Box::new(importer));
    }

    /// Returns metadata for registered importers in registration order.
    pub fn metadata(&self) -> impl Iterator<Item = ImporterMetadata> + '_ {
        self.importers.iter().map(|importer| importer.metadata())
    }

    /// Imports with the highest-confidence registered importer.
    ///
    /// # Errors
    /// Returns [`ImportError::NoImporter`] if no importer recognizes the source,
    /// [`ImportError::AmbiguousImporter`] for an equal-confidence match, or wraps
    /// the selected importer's fatal error.
    pub fn import(&self, source: &ImportSource<'_>) -> Result<ImportedJob, ImportError> {
        let detections = self
            .importers
            .iter()
            .map(|importer| (importer, importer.detect(source)))
            .collect::<Vec<_>>();
        let highest = detections
            .iter()
            .map(|(_, detection)| *detection)
            .max()
            .filter(|detection| *detection != Detection::NotRecognized)
            .ok_or(ImportError::NoImporter)?;
        let matches = detections
            .iter()
            .filter(|(_, detection)| *detection == highest)
            .collect::<Vec<_>>();
        if matches.len() != 1 {
            let format_ids = matches
                .iter()
                .map(|(importer, _)| importer.metadata().format_id)
                .collect::<Vec<_>>()
                .join(", ");
            return Err(ImportError::AmbiguousImporter { format_ids });
        }
        let importer = matches[0].0;

        let metadata = importer.metadata();
        let mut imported = importer
            .import(source)
            .map_err(|source| ImportError::Importer {
                format_id: metadata.format_id,
                source: Box::new(source),
            })?;
        imported.job.source.provenance = metadata.provenance();
        Ok(imported)
    }
}

use librepat_core::{
    Detection, ImportError, ImportProvenance, ImportSource, ImportedJob, ImporterMetadata,
    ImporterRegistry, Job, JobMetadata, SourceArchive, TesterImporter,
};
use time::OffsetDateTime;

const SYNTHETIC_METADATA: ImporterMetadata = ImporterMetadata::new(
    "synthetic-importer",
    "synthetic-format",
    "Synthetic tester export",
    &["synthetic"],
);

struct SyntheticImporter;

impl TesterImporter for SyntheticImporter {
    fn metadata(&self) -> ImporterMetadata {
        SYNTHETIC_METADATA
    }

    fn detect(&self, source: &ImportSource<'_>) -> Detection {
        if source.bytes.starts_with(b"SYNTHETIC FORMAT") {
            Detection::Certain
        } else {
            Detection::NotRecognized
        }
    }

    fn import(&self, source: &ImportSource<'_>) -> Result<ImportedJob, ImportError> {
        if self.detect(source) == Detection::NotRecognized {
            return Err(ImportError::NotRecognized);
        }
        let metadata = JobMetadata {
            site_name: "Synthetic adapter".into(),
            ..JobMetadata::default()
        };
        Ok(ImportedJob {
            job: Job {
                metadata: metadata.clone(),
                original_metadata: metadata,
                appliances: Vec::new(),
                source: SourceArchive {
                    provenance: ImportProvenance {
                        importer_id: "unstamped".into(),
                        format_id: "unstamped".into(),
                    },
                    filename: source.filename.into(),
                    bytes: source.bytes.into(),
                    sha256: [0; 32],
                    imported_at: OffsetDateTime::UNIX_EPOCH,
                    tester_model: String::new(),
                    tester_serial: String::new(),
                },
            },
            warnings: Vec::new(),
        })
    }
}

#[test]
fn registry_should_select_between_fluke_and_synthetic_importers() {
    let mut registry = ImporterRegistry::new();
    librepat_fluke::register(&mut registry);
    registry.register(SyntheticImporter);

    let synthetic = registry
        .import(&ImportSource::new(
            "source.synthetic",
            b"SYNTHETIC FORMAT\n",
        ))
        .unwrap_or_else(|error| panic!("synthetic import failed: {error}"));
    assert_eq!(synthetic.job.metadata.site_name, "Synthetic adapter");
    assert_eq!(
        synthetic.job.source.provenance,
        SYNTHETIC_METADATA.provenance()
    );

    let fluke = registry
        .import(&ImportSource::new(
            "source.flk",
            b"MODEL  SYNTHETIC\nTEST NUMBER  1\nAPP NO  A-001\nVISUAL CHECK  P\nEND OF DATA\n",
        ))
        .unwrap_or_else(|error| panic!("Fluke import failed: {error}"));
    assert_eq!(
        fluke.job.source.provenance,
        librepat_fluke::IMPORTER_METADATA.provenance()
    );
}

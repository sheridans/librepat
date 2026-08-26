use librepat_core::{ImportError, ImportSource, ImporterRegistry};

#[test]
fn registry_should_select_each_labelled_ascii_adapter() {
    let mut registry = ImporterRegistry::new();
    librepat_fluke::register(&mut registry);
    librepat_seaward::register(&mut registry);
    librepat_kewtech::register(&mut registry);

    let cases = [
        (
            "source.flk",
            b"MODEL 6500\nTEST NUMBER 1\nAPP NO A-1\nVISUAL CHECK P\nEND OF DATA\n".as_slice(),
            "fluke-flk-text",
        ),
        (
            "source.txt",
            b"TESTER PrimeTest 350\nTEST NUMBER 1\nTIME 09:30\nAPP NO A-1\nTEST MODE AUTO\nIEC P\n"
                .as_slice(),
            "seaward-primetest-ascii",
        ),
        (
            "source.asc",
            b"TEST NUMBER 1\nAPP NO A-1\nDESCRIPTION Item\nLOCN Room\nVISUAL CHECK P\n".as_slice(),
            "kewtech-kt74-77-ascii",
        ),
    ];
    for (filename, bytes, format_id) in cases {
        let imported = registry
            .import(&ImportSource::new(filename, bytes))
            .unwrap_or_else(|error| panic!("{filename} import failed: {error}"));
        assert_eq!(imported.job.source.provenance.format_id, format_id);
    }
}

#[test]
fn registry_should_report_equal_confidence_as_ambiguous() {
    let mut registry = ImporterRegistry::new();
    registry.register(PossibleImporter);
    registry.register(PossibleImporter);
    let error = match registry.import(&ImportSource::new("source.txt", b"match")) {
        Ok(_) => panic!("equal confidence selected by registration order"),
        Err(error) => error,
    };
    assert!(matches!(error, ImportError::AmbiguousImporter { .. }));
}

#[derive(Clone, Copy)]
struct PossibleImporter;

impl librepat_core::TesterImporter for PossibleImporter {
    fn metadata(&self) -> librepat_core::ImporterMetadata {
        librepat_core::ImporterMetadata::new("test", "ambiguous-test", "Test", &[])
    }

    fn detect(&self, _source: &ImportSource<'_>) -> librepat_core::Detection {
        librepat_core::Detection::Possible
    }

    fn import(
        &self,
        _source: &ImportSource<'_>,
    ) -> Result<librepat_core::ImportedJob, ImportError> {
        Err(ImportError::NotRecognized)
    }
}

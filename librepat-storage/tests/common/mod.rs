use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use librepat_core::{
    Appliance, ApplianceStatus, Comparison, ImportProvenance, Job, JobMetadata, Measurement,
    SourceArchive, TestKind, TestMode, TestResult, TestStatus,
};
use librepat_storage::JobStore;
use sha2::{Digest, Sha256};
use time::{OffsetDateTime, macros::date};

static NEXT_FILE: AtomicU64 = AtomicU64::new(1);

pub struct TestFile(PathBuf);

impl TestFile {
    pub fn new(label: &str) -> Self {
        let sequence = NEXT_FILE.fetch_add(1, Ordering::Relaxed);
        Self(std::env::temp_dir().join(format!(
            "librepat-{label}-{}-{sequence}.librepat",
            std::process::id()
        )))
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TestFile {
    fn drop(&mut self) {
        let _result = fs::remove_file(&self.0);
        let _result = fs::remove_file(self.0.with_extension("librepat-journal"));
    }
}

pub fn sample_job() -> Job {
    let source_bytes = b"synthetic tester export\n".to_vec();
    let source_hash = Sha256::digest(&source_bytes).into();
    let metadata = JobMetadata {
        site_name: "Example site".into(),
        tester_name: "Test operator".into(),
        ..JobMetadata::default()
    };
    Job {
        metadata: metadata.clone(),
        original_metadata: metadata,
        appliances: vec![Appliance {
            source_record_number: "1".into(),
            appliance_id: "A-001".into(),
            description: "Synthetic appliance".into(),
            description_segments: vec!["Synthetic".into(), "appliance".into(), String::new()],
            location: "Test room".into(),
            location_segments: vec!["Test".into(), "room".into()],
            test_date: Some(date!(2026 - 01 - 02)),
            test_time: Some(time::macros::time!(09:30:15)),
            retest_date: None,
            comments: "Synthetic comment".into(),
            mode: TestMode {
                code: "0".into(),
                label: "MAN".into(),
            },
            user: "Test operator".into(),
            tests: vec![
                TestResult {
                    kind: TestKind::EarthBond,
                    raw_name: "EARTH".into(),
                    status: TestStatus::Pass,
                    measurement: Some(Measurement {
                        comparison: Comparison::Equal,
                        value: "0.04".into(),
                        unit: "OHM".into(),
                    }),
                    limit: None,
                    raw_line: "EARTH   0.04 OHM P".into(),
                },
                TestResult {
                    kind: TestKind::IecLead,
                    raw_name: "IEC".into(),
                    status: TestStatus::Pass,
                    measurement: None,
                    limit: None,
                    raw_line: "IEC P".into(),
                },
            ],
            status: ApplianceStatus::Pass,
            raw_fields: Vec::new(),
        }],
        source: SourceArchive {
            provenance: ImportProvenance {
                importer_id: "synthetic-importer".into(),
                format_id: "synthetic-format".into(),
            },
            filename: "synthetic.flk".into(),
            bytes: source_bytes,
            sha256: source_hash,
            imported_at: OffsetDateTime::UNIX_EPOCH,
            tester_model: "SYNTHETIC".into(),
            tester_serial: "0001".into(),
        },
    }
}

pub fn create_store(file: &TestFile) -> JobStore {
    JobStore::create(file.path(), &sample_job())
        .unwrap_or_else(|error| panic!("could not create test job: {error}"))
}

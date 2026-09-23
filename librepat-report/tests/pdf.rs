use librepat_core::{
    Appliance, ApplianceStatus, Comparison, ImportProvenance, Job, JobMetadata, Limit, Measurement,
    SourceArchive, TestKind, TestMode, TestResult, TestStatus,
};
use librepat_report::{generate_appliance_report, summarize_job};
use lopdf::Document;
use time::{OffsetDateTime, macros::date};

#[path = "pdf/certificate.rs"]
mod certificate;

pub(crate) fn sample_job() -> Job {
    let mut metadata = JobMetadata {
        customer_name: "Current customer".into(),
        site_name: "Synthetic test site".into(),
        contractor_name: "Example contractor".into(),
        tester_name: "Test operator".into(),
        certificate_number: "CERT-001".into(),
        ..JobMetadata::default()
    };
    metadata.site_address.town = "Example town".into();
    let appliance = Appliance {
        source_record_number: "1".into(),
        removed: false,
        appliance_id: "A-001".into(),
        description: "Synthetic control panel".into(),
        description_segments: vec!["Synthetic control panel".into()],
        location: "Plant room".into(),
        location_segments: vec!["Plant room".into()],
        test_date: Some(date!(2026 - 02 - 03)),
        test_time: None,
        retest_date: Some(date!(2027 - 02 - 03)),
        comments: String::new(),
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
                raw_line: "EARTH 0.04 OHM P".into(),
            },
            TestResult {
                kind: TestKind::Pelv,
                raw_name: "PROBE PELV".into(),
                status: TestStatus::Skipped,
                measurement: Some(Measurement {
                    comparison: Comparison::Equal,
                    value: "47".into(),
                    unit: "V".into(),
                }),
                limit: Some(Limit {
                    comparison: Comparison::Equal,
                    value: "50".into(),
                    unit: "V".into(),
                    raw: "50 V".into(),
                }),
                raw_line: "PROBE PELV 47 V P".into(),
            },
        ],
        status: ApplianceStatus::Pass,
        raw_fields: Vec::new(),
    };
    let appliances = (1..=25)
        .map(|index| Appliance {
            appliance_id: format!("A-{index:03}"),
            source_record_number: index.to_string(),
            ..appliance.clone()
        })
        .collect();
    Job {
        metadata,
        original_metadata: JobMetadata::default(),
        appliances,
        source: SourceArchive {
            provenance: ImportProvenance::new("synthetic-importer", "synthetic-format"),
            filename: "synthetic.flk".into(),
            bytes: b"synthetic".to_vec(),
            sha256: [0; 32],
            imported_at: OffsetDateTime::UNIX_EPOCH,
            tester_model: "SYNTHETIC".into(),
            tester_serial: "0001".into(),
        },
    }
}

pub(crate) fn parsed_text(bytes: &[u8]) -> (usize, String) {
    let document = Document::load_mem(bytes)
        .unwrap_or_else(|error| panic!("generated PDF is invalid: {error}"));
    let page_numbers = document.get_pages().keys().copied().collect::<Vec<_>>();
    let text = document
        .extract_text(&page_numbers)
        .unwrap_or_else(|error| panic!("could not extract generated PDF text: {error}"));
    (page_numbers.len(), text)
}

fn first_page_is_portrait(bytes: &[u8]) -> bool {
    let Ok(document) = Document::load_mem(bytes) else {
        return false;
    };
    let Some(page_id) = document.get_pages().into_values().next() else {
        return false;
    };
    let Ok(page) = document.get_dictionary(page_id) else {
        return false;
    };
    let Ok(media_box) = page.get(b"MediaBox").and_then(lopdf::Object::as_array) else {
        return false;
    };
    let [left, bottom, right, top] = media_box.as_slice() else {
        return false;
    };
    let Ok(width) = right
        .as_float()
        .and_then(|right| left.as_float().map(|left| right - left))
    else {
        return false;
    };
    let Ok(height) = top
        .as_float()
        .and_then(|top| bottom.as_float().map(|bottom| top - bottom))
    else {
        return false;
    };
    width < height
}

#[test]
fn appliance_report_should_include_final_tests_without_an_annex() {
    let bytes = generate_appliance_report(&sample_job())
        .unwrap_or_else(|error| panic!("could not generate appliance report: {error}"));
    let (pages, text) = parsed_text(&bytes);
    assert_eq!(pages, 2);
    assert!(
        first_page_is_portrait(&bytes),
        "report page is not portrait"
    );
    assert!(
        text.contains("Appliance test results"),
        "missing report header: {text}"
    );
    assert!(
        !text.contains("Test details annex"),
        "unexpected annex: {text}"
    );
    assert!(text.contains("PROBE PELV"), "missing PELV entry: {text}");
    assert!(text.contains("SKIPPED"), "missing PELV outcome: {text}");
    assert!(text.contains("limit 50 V"), "missing PELV limit: {text}");
    assert!(text.contains("Plant room"), "missing location: {text}");
    assert!(text.contains("Page 1 of 2"), "missing footer: {text}");
}

#[test]
fn appliance_report_should_sort_numeric_ids_naturally() {
    let mut job = sample_job();
    let appliance = job.appliances[0].clone();
    job.appliances = [("10", "Tenth"), ("2", "Second"), ("1", "First")]
        .into_iter()
        .map(|(id, description)| Appliance {
            appliance_id: id.into(),
            description: description.into(),
            ..appliance.clone()
        })
        .collect();
    let bytes = generate_appliance_report(&job)
        .unwrap_or_else(|error| panic!("could not generate appliance report: {error}"));
    let (_, text) = parsed_text(&bytes);
    let first = text
        .find("First")
        .unwrap_or_else(|| panic!("missing first appliance: {text}"));
    let second = text
        .find("Second")
        .unwrap_or_else(|| panic!("missing second appliance: {text}"));
    let tenth = text
        .find("Tenth")
        .unwrap_or_else(|| panic!("missing tenth appliance: {text}"));
    assert!(first < second && second < tenth, "wrong ID order: {text}");
}

#[test]
fn appliance_report_should_use_the_final_repeated_test() {
    let mut job = sample_job();
    job.appliances.truncate(1);
    let mut final_earth = job.appliances[0].tests[0].clone();
    job.appliances[0].tests[0].status = TestStatus::Fail;
    job.appliances[0].tests[0]
        .measurement
        .as_mut()
        .unwrap_or_else(|| panic!("synthetic earth result has no reading"))
        .value = "9.99".into();
    final_earth.status = TestStatus::Pass;
    final_earth
        .measurement
        .as_mut()
        .unwrap_or_else(|| panic!("synthetic earth result has no reading"))
        .value = "0.04".into();
    job.appliances[0].tests.push(final_earth);
    job.appliances[0].status = ApplianceStatus::Pass;
    let bytes = generate_appliance_report(&job)
        .unwrap_or_else(|error| panic!("could not generate appliance report: {error}"));
    let (_, text) = parsed_text(&bytes);
    assert!(
        text.contains("0.04") && !text.contains("9.99"),
        "report did not select the final result: {text}"
    );
}

#[test]
fn appliance_report_should_omit_removed_appliances() {
    let mut job = sample_job();
    job.appliances[0].removed = true;

    let bytes = generate_appliance_report(&job)
        .unwrap_or_else(|error| panic!("could not generate appliance report: {error}"));
    let (_, text) = parsed_text(&bytes);

    assert!(
        text.contains("24 appliances") && !text.contains("A-001"),
        "removed appliance appeared in report: {text}"
    );
}

#[test]
fn job_summary_should_omit_removed_appliances() {
    let mut job = sample_job();
    job.appliances[0].removed = true;

    let summary = summarize_job(&job);

    assert_eq!(
        (summary.appliances, summary.passed, summary.failed),
        (24, 24, 0)
    );
}

use librepat_core::ReportDateFormat;
use librepat_report::generate_completion_certificate;
use time::macros::date;

use super::{parsed_text, sample_job};

#[test]
fn certificate_should_use_current_metadata_and_expected_summary() {
    let bytes = generate_completion_certificate(&sample_job())
        .unwrap_or_else(|error| panic!("could not generate certificate: {error}"));
    let (pages, text) = parsed_text(&bytes);
    assert_eq!(pages, 1);
    assert!(
        text.contains("Completion certificate"),
        "missing title: {text}"
    );
    assert!(
        text.contains("Current customer"),
        "missing edited customer: {text}"
    );
    assert!(text.contains("25"), "missing appliance total: {text}");
    assert!(
        text.contains("Test operator") && !text.contains("TESTER"),
        "tester should appear only as the declaration name: {text}"
    );
    assert!(
        text.contains("Name")
            && text.contains("Date")
            && !text.contains("Signature")
            && !text.contains("Signed date")
            && !text.contains("NO RESULT"),
        "certificate declaration name/date fields did not match: {text}"
    );
    assert!(
        !text.contains("ORDER REFERENCE") && !text.contains("TELEPHONE") && !text.contains("EMAIL"),
        "blank optional fields are visible: {text}"
    );
    assert!(
        !text.contains("VALID UNTIL") && !text.contains("Not specified"),
        "unset validity should be omitted: {text}"
    );
}

#[test]
fn certificate_should_render_postal_address_lines_without_truncation() {
    let mut job = sample_job();
    job.metadata.contractor_address.line_1 = "Example House".into();
    job.metadata.contractor_address.line_2 = "Long Industrial Estate Road".into();
    job.metadata.contractor_address.town = "Exampleton".into();
    job.metadata.contractor_address.postcode = "EX1 2AB".into();
    let bytes = generate_completion_certificate(&job)
        .unwrap_or_else(|error| panic!("could not generate certificate: {error}"));
    let (_, text) = parsed_text(&bytes);
    assert!(
        text.contains("Long Industrial Estate Road") && !text.contains('…'),
        "address was truncated: {text}"
    );
}

#[test]
fn certificate_should_use_saved_numeric_date_format() {
    let mut job = sample_job();
    job.metadata.report_date_format = ReportDateFormat::DayMonthYear;
    job.metadata.valid_until = Some(date!(2027 - 02 - 03));
    let bytes = generate_completion_certificate(&job)
        .unwrap_or_else(|error| panic!("could not generate certificate: {error}"));
    let (_, text) = parsed_text(&bytes);
    assert!(
        text.contains("03/02/2027"),
        "missing formatted date: {text}"
    );
}

#[test]
fn certificate_should_show_valid_until_only_when_explicitly_set() {
    let mut job = sample_job();
    job.metadata.valid_until = Some(date!(2027 - 02 - 03));
    let bytes = generate_completion_certificate(&job)
        .unwrap_or_else(|error| panic!("could not generate certificate: {error}"));
    let (_, text) = parsed_text(&bytes);
    assert!(
        text.contains("VALID UNTIL") && text.contains("03 Feb 2027"),
        "explicit validity is missing: {text}"
    );
}

#[test]
fn certificate_should_omit_internal_provenance_note() {
    let bytes = generate_completion_certificate(&sample_job())
        .unwrap_or_else(|error| panic!("could not generate certificate: {error}"));
    let (_, text) = parsed_text(&bytes);
    assert!(
        !text.contains("Results remain as imported"),
        "internal note leaked: {text}"
    );
}

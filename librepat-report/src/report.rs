use librepat_core::{Appliance, Job, latest_tests};
use printpdf::{PdfDocument, PdfSaveOptions};

use crate::{
    ReportError,
    canvas::{INK, MUTED, PageCanvas, TEAL, WHITE, finish_pages, truncate},
    fonts::ReportFonts,
    report_data::{
        blank_as_dash, format_date, sorted_appliances, status_color, status_text,
        test_status_color, test_summary,
    },
};

const FIRST_ROW_Y: f32 = 237.0;
const BOTTOM_Y: f32 = 18.0;
const TESTS_PER_LINE: usize = 3;
const TEST_LINE_HEIGHT: f32 = 5.5;

/// Generates an original A4 portrait appliance results report.
///
/// Each appliance appears once in natural ID order. Repeated tests use their
/// final imported occurrence while every source result remains stored in the job.
///
/// # Errors
/// Returns an error if the embedded fonts cannot be parsed.
pub fn generate_appliance_report(job: &Job) -> Result<Vec<u8>, ReportError> {
    let mut document = PdfDocument::new("LibrePAT appliance results");
    let fonts = ReportFonts::add_to(&mut document)?;
    let pages = report_pages(job, &fonts);
    document.with_pages(finish_pages(pages));
    Ok(document.save(&PdfSaveOptions::default(), &mut Vec::new()))
}

fn report_pages(job: &Job, fonts: &ReportFonts) -> Vec<PageCanvas> {
    let mut pages = Vec::new();
    let mut page = report_page(job, fonts);
    let mut y = FIRST_ROW_Y;
    for appliance in sorted_appliances(job) {
        let tests = latest_tests(&appliance.tests);
        let height = appliance_height(tests.len());
        if y - height < BOTTOM_Y && y < FIRST_ROW_Y {
            pages.push(page);
            page = report_page(job, fonts);
            y = FIRST_ROW_Y;
        }
        draw_appliance(&mut page, job, appliance, &tests, y);
        y -= height;
    }
    pages.push(page);
    pages
}

fn report_page(job: &Job, fonts: &ReportFonts) -> PageCanvas {
    let mut page = PageCanvas::new(210.0, 297.0, fonts);
    page.rect(0.0, 264.0, 210.0, 33.0, TEAL);
    page.text(12.0, 281.0, 19.0, true, WHITE, "Appliance test results");
    page.text(
        12.0,
        271.0,
        9.0,
        false,
        WHITE,
        truncate(&job.metadata.site_name, 52),
    );
    page.text(
        164.0,
        271.0,
        7.5,
        false,
        WHITE,
        format!("{} appliances", job.appliances.len()),
    );
    page.text(
        12.0,
        256.0,
        7.5,
        false,
        MUTED,
        format!(
            "Certificate {}",
            blank_as_dash(&job.metadata.certificate_number)
        ),
    );
    table_header(&mut page);
    page
}

fn table_header(page: &mut PageCanvas) {
    page.rect(12.0, 242.0, 186.0, 8.0, (0.84, 0.86, 0.82));
    for (x, label) in [
        (14.0, "ID"),
        (35.0, "Description"),
        (87.0, "Location"),
        (122.0, "Test date"),
        (146.0, "Retest"),
        (176.0, "Status"),
    ] {
        page.text(x, 245.0, 6.2, true, INK, label);
    }
}

fn appliance_height(test_count: usize) -> f32 {
    let test_lines = test_count.max(1).div_ceil(TESTS_PER_LINE);
    9.0 + test_lines as f32 * TEST_LINE_HEIGHT
}

fn draw_appliance(
    page: &mut PageCanvas,
    job: &Job,
    appliance: &Appliance,
    tests: &[&librepat_core::TestResult],
    y: f32,
) {
    page.rule(12.0, y + 3.0, 186.0, (0.82, 0.82, 0.78));
    page.text(
        14.0,
        y,
        6.5,
        true,
        INK,
        truncate(&appliance.appliance_id, 12),
    );
    page.text(
        35.0,
        y,
        6.5,
        false,
        INK,
        truncate(&appliance.description, 27),
    );
    page.text(87.0, y, 6.5, false, INK, truncate(&appliance.location, 18));
    page.text(
        122.0,
        y,
        5.8,
        false,
        INK,
        format_date(appliance.test_date, job.metadata.report_date_format),
    );
    page.text(
        146.0,
        y,
        5.8,
        false,
        INK,
        format_date(appliance.retest_date, job.metadata.report_date_format),
    );
    page.text(
        176.0,
        y,
        6.2,
        true,
        status_color(appliance.status),
        status_text(appliance.status),
    );
    draw_tests(page, tests, y - 6.0);
}

fn draw_tests(page: &mut PageCanvas, tests: &[&librepat_core::TestResult], start_y: f32) {
    if tests.is_empty() {
        page.text(14.0, start_y, 5.7, false, MUTED, "No imported test results");
        return;
    }
    for (line, chunk) in tests.chunks(TESTS_PER_LINE).enumerate() {
        let y = start_y - line as f32 * TEST_LINE_HEIGHT;
        for (column, test) in chunk.iter().enumerate() {
            page.text(
                14.0 + column as f32 * 61.0,
                y,
                5.4,
                false,
                test_status_color(test.status),
                truncate(&test_summary(test), 48),
            );
        }
    }
}

use librepat_core::{Job, PostalAddress, ReportDateFormat};
use printpdf::{PdfDocument, PdfSaveOptions};
use time::{Date, OffsetDateTime, UtcOffset};

use crate::{
    ReportError,
    canvas::{INK, MUTED, PageCanvas, TEAL, WHITE, finish_pages, truncate},
    fonts::ReportFonts,
    summarize_job,
};

/// Generates an original A4 portrait completion certificate.
///
/// # Errors
/// Returns an error if the embedded fonts cannot be parsed.
pub fn generate_completion_certificate(job: &Job) -> Result<Vec<u8>, ReportError> {
    let mut document = PdfDocument::new("LibrePAT completion certificate");
    let fonts = ReportFonts::add_to(&mut document)?;
    let mut page = PageCanvas::new(210.0, 297.0, &fonts);
    page.rect(0.0, 249.0, 210.0, 48.0, TEAL);
    page.text(15.0, 276.0, 22.0, true, WHITE, "Completion certificate");
    page.text(
        15.0,
        261.0,
        9.0,
        false,
        WHITE,
        "Portable appliance inspection and testing",
    );
    page.text(
        155.0,
        261.0,
        8.0,
        false,
        WHITE,
        truncate(&job.metadata.certificate_number, 24),
    );

    section_title(&mut page, 235.0, "CUSTOMER & SITE");
    labelled_value(
        &mut page,
        15.0,
        224.0,
        "Customer",
        &job.metadata.customer_name,
    );
    labelled_value(&mut page, 110.0, 224.0, "Site", &job.metadata.site_name);
    address_block(&mut page, 15.0, 211.0, &job.metadata.customer_address);
    address_block(&mut page, 110.0, 211.0, &job.metadata.site_address);

    let summary = summarize_job(job);
    section_title(&mut page, 184.0, "COMPLETION SUMMARY");
    stat_box(
        &mut page,
        15.0,
        157.0,
        "APPLIANCES",
        summary.appliances,
        TEAL,
    );
    stat_box(
        &mut page,
        77.0,
        157.0,
        "PASSED",
        summary.passed,
        (0.04, 0.46, 0.34),
    );
    stat_box(
        &mut page,
        139.0,
        157.0,
        "FAILED",
        summary.failed,
        (0.68, 0.12, 0.10),
    );
    let date_format = job.metadata.report_date_format;
    let testing_period = match (summary.first_test, summary.last_test) {
        (Some(first), Some(last)) if first != last => {
            format!(
                "{} to {}",
                date_format.format(first),
                date_format.format(last)
            )
        }
        (Some(date), _) => date_format.format(date),
        _ => "Not recorded".into(),
    };
    labelled_value(&mut page, 15.0, 143.0, "Testing period", &testing_period);
    if let Some(valid_until) = job.metadata.valid_until {
        labelled_value(
            &mut page,
            110.0,
            143.0,
            "Valid until",
            &date_format.format(valid_until),
        );
    }

    section_title(&mut page, 123.0, "TESTING ORGANISATION");
    labelled_value(
        &mut page,
        15.0,
        112.0,
        "Contractor",
        &job.metadata.contractor_name,
    );
    address_block(&mut page, 15.0, 99.0, &job.metadata.contractor_address);
    optional_labelled_value(
        &mut page,
        110.0,
        99.0,
        "Order reference",
        &job.metadata.order_reference,
    );
    optional_labelled_value(
        &mut page,
        110.0,
        87.0,
        "Telephone",
        &job.metadata.contractor_telephone,
    );
    optional_labelled_value(
        &mut page,
        110.0,
        75.0,
        "Email",
        &job.metadata.contractor_email,
    );

    section_title(&mut page, 59.0, "DECLARATION");
    page.text(
        15.0,
        48.0,
        8.5,
        false,
        INK,
        "The appliances listed in the accompanying report were inspected and tested.",
    );
    page.rule(15.0, 29.0, 78.0, (0.32, 0.38, 0.38));
    if !job.metadata.tester_name.trim().is_empty() {
        page.text(
            15.0,
            31.0,
            8.0,
            false,
            INK,
            truncate(&job.metadata.tester_name, 42),
        );
    }
    page.text(15.0, 24.0, 7.0, false, MUTED, "Name");
    page.rule(110.0, 29.0, 48.0, (0.32, 0.38, 0.38));
    page.text(
        110.0,
        31.0,
        8.0,
        false,
        INK,
        declaration_date_text(job.metadata.report_date_format, local_calendar_date()),
    );
    page.text(110.0, 24.0, 7.0, false, MUTED, "Date");

    document.with_pages(finish_pages(vec![page]));
    Ok(document.save(&PdfSaveOptions::default(), &mut Vec::new()))
}

fn section_title(page: &mut PageCanvas, y: f32, title: &str) {
    page.text(15.0, y, 8.0, true, TEAL, title);
    page.rule(15.0, y - 4.0, 180.0, (0.72, 0.77, 0.73));
}

fn labelled_value(page: &mut PageCanvas, x: f32, y: f32, label: &str, value: &str) {
    page.text(x, y, 6.5, true, MUTED, label.to_ascii_uppercase());
    page.text(x, y - 6.0, 9.0, true, INK, truncate(value, 42));
}

fn optional_labelled_value(page: &mut PageCanvas, x: f32, y: f32, label: &str, value: &str) {
    if !value.trim().is_empty() {
        labelled_value(page, x, y, label, value);
    }
}

fn address_block(page: &mut PageCanvas, x: f32, y: f32, address: &PostalAddress) {
    let lines = [
        address.line_1.as_str(),
        address.line_2.as_str(),
        address.town.as_str(),
        address.county.as_str(),
        address.postcode.as_str(),
    ];
    for (index, line) in lines
        .into_iter()
        .filter(|line| !line.trim().is_empty())
        .enumerate()
    {
        page.text(x, y - index as f32 * 4.5, 7.5, false, MUTED, line);
    }
}

fn stat_box(
    page: &mut PageCanvas,
    x: f32,
    y: f32,
    label: &str,
    value: usize,
    accent: (f32, f32, f32),
) {
    page.rect(x, y, 56.0, 20.0, (0.90, 0.90, 0.86));
    page.rect(x, y, 2.0, 20.0, accent);
    page.text(x + 5.0, y + 11.0, 13.0, true, INK, value.to_string());
    page.text(x + 5.0, y + 4.0, 6.0, true, MUTED, label);
}

fn local_calendar_date() -> Date {
    let now = OffsetDateTime::now_utc();
    calendar_date_at_offset(now, UtcOffset::local_offset_at(now).ok())
}

fn calendar_date_at_offset(now: OffsetDateTime, offset: Option<UtcOffset>) -> Date {
    now.to_offset(offset.unwrap_or(UtcOffset::UTC)).date()
}

fn declaration_date_text(format: ReportDateFormat, date: Date) -> String {
    format.format(date)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_calendar_date_should_apply_the_available_offset() {
        let now = time::macros::datetime!(2026-08-24 23:30 UTC);
        assert_eq!(
            calendar_date_at_offset(now, Some(time::macros::offset!(+1))),
            time::macros::date!(2026 - 08 - 25)
        );
    }

    #[test]
    fn local_calendar_date_should_fall_back_to_utc_when_offset_is_unavailable() {
        let now = time::macros::datetime!(2026-08-24 23:30 UTC);
        assert_eq!(
            calendar_date_at_offset(now, None),
            time::macros::date!(2026 - 08 - 24)
        );
    }

    #[test]
    fn declaration_date_should_use_the_selected_format() {
        assert_eq!(
            declaration_date_text(
                ReportDateFormat::DayMonthYear,
                time::macros::date!(2026 - 08 - 24),
            ),
            "24/08/2026"
        );
    }
}

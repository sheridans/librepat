//! Original PDF report and completion certificate generation.

mod canvas;
mod certificate;
mod error;
mod fonts;
mod report;
mod report_data;
mod summary;

use std::{fs, path::Path};

pub use certificate::generate_completion_certificate;
pub use error::ReportError;
pub use report::generate_appliance_report;
pub use summary::{JobSummary, summarize_job};

/// Writes generated PDF bytes to a selected destination.
///
/// # Errors
/// Returns an error if the destination cannot be written.
pub fn save_pdf(path: impl AsRef<Path>, bytes: &[u8]) -> Result<(), ReportError> {
    fs::write(path, bytes)?;
    Ok(())
}

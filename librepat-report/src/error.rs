use thiserror::Error;

/// PDF font parsing or file output failure.
#[derive(Debug, Error)]
pub enum ReportError {
    #[error("embedded Go font could not be parsed")]
    InvalidFont,
    #[error("could not write PDF: {0}")]
    Io(#[from] std::io::Error),
}

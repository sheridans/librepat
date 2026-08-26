use printpdf::{ParsedFont, PdfDocument, PdfFontHandle};

use crate::ReportError;

const GO_REGULAR: &[u8] = include_bytes!("../../assets/fonts/Go-Regular.ttf");
const GO_BOLD: &[u8] = include_bytes!("../../assets/fonts/Go-Bold.ttf");

#[derive(Clone)]
pub(crate) struct ReportFonts {
    pub regular: PdfFontHandle,
    pub bold: PdfFontHandle,
}

impl ReportFonts {
    pub fn add_to(document: &mut PdfDocument) -> Result<Self, ReportError> {
        let regular = ParsedFont::from_bytes(GO_REGULAR, 0, &mut Vec::new())
            .ok_or(ReportError::InvalidFont)?;
        let bold =
            ParsedFont::from_bytes(GO_BOLD, 0, &mut Vec::new()).ok_or(ReportError::InvalidFont)?;
        Ok(Self {
            regular: PdfFontHandle::External(document.add_font(&regular)),
            bold: PdfFontHandle::External(document.add_font(&bold)),
        })
    }
}

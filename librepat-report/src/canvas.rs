use printpdf::{Color, Mm, Op, PaintMode, PdfPage, Point, Pt, Rect, Rgb, TextItem};

use crate::fonts::ReportFonts;

pub(crate) const INK: (f32, f32, f32) = (0.08, 0.13, 0.16);
pub(crate) const MUTED: (f32, f32, f32) = (0.38, 0.44, 0.46);
pub(crate) const PAPER: (f32, f32, f32) = (0.97, 0.96, 0.92);
pub(crate) const TEAL: (f32, f32, f32) = (0.02, 0.36, 0.38);
pub(crate) const WHITE: (f32, f32, f32) = (1.0, 1.0, 1.0);

pub(crate) struct PageCanvas {
    width: f32,
    height: f32,
    fonts: ReportFonts,
    ops: Vec<Op>,
}

impl PageCanvas {
    pub fn new(width: f32, height: f32, fonts: &ReportFonts) -> Self {
        let mut canvas = Self {
            width,
            height,
            fonts: fonts.clone(),
            ops: Vec::new(),
        };
        canvas.rect(0.0, 0.0, width, height, PAPER);
        canvas
    }

    pub fn text(
        &mut self,
        x: f32,
        y: f32,
        size: f32,
        bold: bool,
        color: (f32, f32, f32),
        value: impl Into<String>,
    ) {
        let font = if bold {
            self.fonts.bold.clone()
        } else {
            self.fonts.regular.clone()
        };
        self.ops.extend([
            Op::StartTextSection,
            Op::SetTextCursor {
                pos: Point::new(Mm(x), Mm(y)),
            },
            Op::SetFont {
                font,
                size: Pt(size),
            },
            Op::SetFillColor { col: rgb(color) },
            Op::ShowText {
                items: vec![TextItem::Text(value.into())],
            },
            Op::EndTextSection,
        ]);
    }

    pub fn rect(&mut self, x: f32, y: f32, width: f32, height: f32, color: (f32, f32, f32)) {
        let x: Pt = Mm(x).into();
        let y: Pt = Mm(y).into();
        let width: Pt = Mm(width).into();
        let height: Pt = Mm(height).into();
        self.ops.extend([
            Op::SetFillColor { col: rgb(color) },
            Op::DrawRectangle {
                rectangle: Rect {
                    x,
                    y,
                    width,
                    height,
                    mode: Some(PaintMode::Fill),
                    winding_order: None,
                },
            },
        ]);
    }

    pub fn rule(&mut self, x: f32, y: f32, width: f32, color: (f32, f32, f32)) {
        self.rect(x, y, width, 0.25, color);
    }

    pub fn footer(&mut self, page: usize, total: usize) {
        self.rule(12.0, 10.5, self.width - 24.0, (0.72, 0.72, 0.68));
        self.text(12.0, 6.5, 7.0, false, MUTED, "LibrePAT");
        self.text(
            self.width - 34.0,
            6.5,
            7.0,
            false,
            MUTED,
            format!("Page {page} of {total}"),
        );
    }

    pub fn into_page(self) -> PdfPage {
        PdfPage::new(Mm(self.width), Mm(self.height), self.ops)
    }
}

pub(crate) fn finish_pages(mut pages: Vec<PageCanvas>) -> Vec<PdfPage> {
    let total = pages.len();
    pages
        .iter_mut()
        .enumerate()
        .for_each(|(index, page)| page.footer(index + 1, total));
    pages.into_iter().map(PageCanvas::into_page).collect()
}

pub(crate) fn truncate(value: &str, maximum: usize) -> String {
    if value.chars().count() <= maximum {
        value.to_owned()
    } else {
        value
            .chars()
            .take(maximum.saturating_sub(1))
            .collect::<String>()
            + "…"
    }
}

fn rgb((red, green, blue): (f32, f32, f32)) -> Color {
    Color::Rgb(Rgb {
        r: red,
        g: green,
        b: blue,
        icc_profile: None,
    })
}

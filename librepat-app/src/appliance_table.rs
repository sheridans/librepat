use eframe::egui::{self, Color32, Response, RichText, Sense, WidgetText};

use crate::{
    session::{
        DetailState, JobSession, SortColumn, TableState, format_display_date, status_text,
        visible_indices,
    },
    theme,
};

const MIN_TABLE_WIDTH: f32 = 840.0;
const COLUMN_GAP: f32 = 8.0;

#[derive(Clone, Copy)]
struct TableWidths {
    select: f32,
    id: f32,
    description: f32,
    location: f32,
    date: f32,
    status: f32,
}

impl TableWidths {
    fn new(available: f32) -> Self {
        let table_width = available.max(MIN_TABLE_WIDTH);
        let fixed_width = 26.0 + 92.0 + 95.0 * 2.0 + 72.0 + COLUMN_GAP * 6.0;
        let flexible_width = table_width - fixed_width;
        Self {
            select: 26.0,
            id: 92.0,
            description: flexible_width * 0.55,
            location: flexible_width * 0.45,
            date: 95.0,
            status: 72.0,
        }
    }

    fn total(self) -> f32 {
        self.select
            + self.id
            + self.description
            + self.location
            + self.date * 2.0
            + self.status
            + COLUMN_GAP * 6.0
    }
}

pub(crate) fn show(ui: &mut egui::Ui, session: &JobSession, table: &mut TableState) {
    let indices = visible_indices(&session.job, table);
    let widths = TableWidths::new(ui.available_width() - ui.spacing().scroll.bar_width);
    let mut open_detail = None;
    egui::ScrollArea::both()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.set_min_width(widths.total());
            ui.spacing_mut().item_spacing.x = COLUMN_GAP;
            table_header(ui, table, widths);
            ui.separator();
            for (row_number, index) in indices.into_iter().enumerate() {
                appliance_row(
                    ui,
                    session,
                    table,
                    widths,
                    row_number,
                    index,
                    &mut open_detail,
                );
            }
        });
    if let Some(index) = open_detail {
        table.detail = Some(DetailState::new(index, &session.job.appliances[index]));
    }
}

fn table_header(ui: &mut egui::Ui, table: &mut TableState, widths: TableWidths) {
    ui.horizontal(|ui| {
        ui.add_sized([widths.select, 24.0], egui::Label::new(""));
        sort_header(ui, table, SortColumn::Id, "ID", widths.id);
        sort_header(
            ui,
            table,
            SortColumn::Description,
            "DESCRIPTION",
            widths.description,
        );
        sort_header(ui, table, SortColumn::Location, "LOCATION", widths.location);
        sort_header(ui, table, SortColumn::TestDate, "TEST DATE", widths.date);
        sort_header(
            ui,
            table,
            SortColumn::RetestDate,
            "RETEST DATE",
            widths.date,
        );
        sort_header(ui, table, SortColumn::Status, "STATUS", widths.status);
    });
}

fn appliance_row(
    ui: &mut egui::Ui,
    session: &JobSession,
    table: &mut TableState,
    widths: TableWidths,
    row_number: usize,
    index: usize,
    open_detail: &mut Option<usize>,
) {
    let appliance = &session.job.appliances[index];
    let selected = table.selection.contains(&index);
    let fill = if selected {
        theme::TEAL_WASH
    } else if row_number % 2 == 1 {
        ui.visuals().faint_bg_color
    } else {
        Color32::TRANSPARENT
    };
    let (checkbox_value, checkbox_changed, response) = egui::Frame::new()
        .fill(fill)
        .inner_margin(egui::Margin::symmetric(0, 2))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let mut checkbox_value = selected;
                let checkbox_changed = ui
                    .add_sized(
                        [widths.select, 24.0],
                        egui::Checkbox::new(&mut checkbox_value, ""),
                    )
                    .changed();
                let mut response = table_cell(ui, widths.id, &appliance.appliance_id);
                response =
                    response.union(table_cell(ui, widths.description, &appliance.description));
                response = response.union(table_cell(ui, widths.location, &appliance.location));
                let date_format = session.job.metadata.report_date_format;
                response = response.union(table_cell(
                    ui,
                    widths.date,
                    format_display_date(appliance.test_date, date_format),
                ));
                response = response.union(table_cell(
                    ui,
                    widths.date,
                    format_display_date(appliance.retest_date, date_format),
                ));
                response = response.union(table_cell(
                    ui,
                    widths.status,
                    RichText::new(status_text(appliance.status))
                        .strong()
                        .color(theme::status_color(appliance.status)),
                ));
                (checkbox_value, checkbox_changed, response)
            })
            .inner
        })
        .inner;
    if checkbox_changed {
        set_selected(table, index, checkbox_value);
    }
    if response.clicked() {
        if ui.input(|input| input.modifiers.command || input.modifiers.ctrl) {
            set_selected(table, index, !table.selection.contains(&index));
        } else {
            table.selection.clear();
            table.selection.insert(index);
        }
    }
    if response.double_clicked() {
        *open_detail = Some(index);
    }
}

fn table_cell(ui: &mut egui::Ui, width: f32, text: impl Into<WidgetText>) -> Response {
    ui.add_sized(
        [width, 24.0],
        egui::Label::new(text).truncate().sense(Sense::click()),
    )
}

fn set_selected(table: &mut TableState, index: usize, selected: bool) {
    if selected {
        table.selection.insert(index);
    } else {
        table.selection.remove(&index);
    }
}

fn sort_header(
    ui: &mut egui::Ui,
    state: &mut TableState,
    column: SortColumn,
    label: &str,
    width: f32,
) {
    let marker = if state.sort == column {
        if state.ascending { " ↑" } else { " ↓" }
    } else {
        ""
    };
    if ui
        .add_sized(
            [width, 26.0],
            egui::Button::new(format!("{label}{marker}")).frame(false),
        )
        .clicked()
    {
        if state.sort == column {
            state.ascending = !state.ascending;
        } else {
            state.sort = column;
            state.ascending = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::table_cell;

    #[test]
    fn table_cell_should_use_requested_width() {
        eframe::egui::__run_test_ui(|ui| {
            let response = table_cell(ui, 240.0, "short");

            assert_eq!(response.rect.width(), 240.0);
        });
    }
}

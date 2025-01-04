use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style, Stylize},
    text::Text,
    widgets::{Cell, HighlightSpacing, Row, Table, TableState},
    Frame,
};

use crate::TableColors;

const HEADERS: [&str; 2] = ["HELLO", "REPORT"];

pub struct ReportPage {
    summary: Vec<Vec<String>>,
    state: TableState,
}

impl ReportPage {
    pub fn new() -> Self {
        ReportPage {
            summary: vec![vec!["hello".to_string(), "report".to_string()]],
            state: TableState::default().with_selected(0),
        }
    }

    pub fn draw(&mut self, frame: &mut Frame, area: Rect, colors: &TableColors) {
        let vertical = &Layout::vertical([Constraint::Min(5), Constraint::Length(3)]);
        let rects = vertical.split(area);

        self.render_table(frame, rects[0], colors);
        self.render_graph(frame, rects[1], colors);
    }

    fn render_table(&mut self, frame: &mut Frame, area: Rect, colors: &TableColors) {
        let header_style = Style::default().fg(colors.header_fg).bg(colors.header_bg);
        let selected_row_style = Style::default()
            .add_modifier(Modifier::REVERSED)
            .fg(colors.selected_row_style_fg);
        let selected_col_style = Style::default().fg(colors.selected_column_style_fg);
        let selected_cell_style = Style::default()
            .add_modifier(Modifier::REVERSED)
            .fg(colors.selected_cell_style_fg);

        let header = HEADERS
            .into_iter()
            .map(|name| Cell::from(name))
            .collect::<Row>()
            .style(header_style)
            .height(1);
        let rows = self.summary.iter().enumerate().map(|(i, row)| {
            let color = match i % 2 {
                0 => colors.normal_row_color,
                _ => colors.alt_row_color,
            };
            let row = Row::new(row.iter().map(|c| c.as_str()));
            row.style(Style::new().fg(colors.row_fg).bg(color))
                .height(3)
        });
        let bar = " █ ";
        let t = Table::new(rows, [9, 9])
            .header(header)
            .row_highlight_style(selected_row_style)
            .column_highlight_style(selected_col_style)
            .cell_highlight_style(selected_cell_style)
            .highlight_symbol(Text::from(vec!["".into(), bar.into(), "".into()]))
            .bg(colors.buffer_bg)
            .highlight_spacing(HighlightSpacing::Always);
        frame.render_stateful_widget(t, area, &mut self.state);
    }

    fn render_graph(&mut self, frame: &mut Frame, area: Rect, colors: &TableColors) {}
}

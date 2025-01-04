use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style, Stylize},
    text::Text,
    widgets::{Cell, HighlightSpacing, Row, Table, TableState},
    Frame,
};

use crate::{transaction::TransactionsReport, TableColors};

pub struct ReportPage {
    summary: TransactionsReport,
    table_state: TableState,
}

impl ReportPage {
    pub fn new() -> Self {
        ReportPage {
            summary: TransactionsReport::new(&vec![]),
            table_state: TableState::default().with_selected(0),
        }
    }

    pub fn reload(&mut self, report: TransactionsReport) {
        self.summary = report;
    }

    fn update_selected(&mut self, i: usize) {
        self.table_state.select(Some(i));
    }

    fn next_row(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= self.summary.rows_len() - 1 {
                    self.summary.rows_len() - 1
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.update_selected(i);
    }

    fn previous_row(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    0
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.update_selected(i);
    }

    fn first_row(&mut self) {
        let i = 0;
        self.update_selected(i);
    }

    fn last_row(&mut self) {
        let i = self.summary.rows_len() - 1;
        self.update_selected(i);
    }

    fn select_first_column(&mut self) {
        self.table_state.select_column(Some(0));
    }

    fn select_last_column(&mut self) {
        self.table_state
            .select_column(Some(self.summary.cols_len() - 1));
    }

    fn next_column(&mut self) {
        if self.table_state.selected_column() == Some(self.summary.cols_len() - 1) {
            self.select_first_column();
            self.next_row();
        } else {
            self.table_state.select_next_column();
        }
    }

    fn previous_column(&mut self) {
        if self.table_state.selected_column() == Some(0) {
            self.select_last_column();
            self.previous_row();
        } else {
            self.table_state.select_previous_column();
        }
    }

    pub fn handle_key_events(&mut self, key: KeyEvent) {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Tab | KeyCode::Right => self.next_column(),
                KeyCode::BackTab | KeyCode::Left => self.previous_column(),
                KeyCode::Down => self.next_row(),
                KeyCode::Up => self.previous_row(),
                KeyCode::PageUp => self.first_row(),
                KeyCode::PageDown => self.last_row(),
                _ => {}
            }
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

        let header = self
            .summary
            .header_row()
            .into_iter()
            .map(|name| Cell::from(name))
            .collect::<Row>()
            .style(header_style)
            .height(1);
        let rows = self
            .summary
            .get_rows()
            .into_iter()
            .enumerate()
            .map(|(i, row)| {
                let color = match i % 2 {
                    0 => colors.normal_row_color,
                    _ => colors.alt_row_color,
                };

                let row = Row::new(row);
                row.style(Style::new().fg(colors.row_fg).bg(color))
                    .height(3)
            });
        let bar = " █ ";
        let t = Table::new(rows, vec![10; 14])
            .header(header)
            .row_highlight_style(selected_row_style)
            .column_highlight_style(selected_col_style)
            .cell_highlight_style(selected_cell_style)
            .highlight_symbol(Text::from(vec!["".into(), bar.into(), "".into()]))
            .bg(colors.buffer_bg)
            .highlight_spacing(HighlightSpacing::Always);
        frame.render_stateful_widget(t, area, &mut self.table_state);
    }

    fn render_graph(&mut self, _frame: &mut Frame, _area: Rect, _colors: &TableColors) {}
}

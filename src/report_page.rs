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
    number_of_columns: usize,
    selected_column_buffer: usize,
}

impl ReportPage {
    pub fn new() -> Self {
        ReportPage {
            summary: TransactionsReport::new(&vec![]),
            table_state: TableState::default().with_selected(0),
            number_of_columns: 14,
            selected_column_buffer: 0,
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
        self.selected_column_buffer = 0;
    }

    fn select_last_column(&mut self) {
        self.table_state
            .select_column(Some(self.number_of_columns - 1));
        self.selected_column_buffer = self.summary.cols_len() - self.number_of_columns + 1;
    }

    fn next_column(&mut self) {
        let table_selected_column = self.table_state.selected_column().unwrap_or(0);
        if self.selected_column_buffer + table_selected_column >= self.summary.cols_len() {
            self.select_first_column();
            self.next_row();
        } else {
            if table_selected_column < self.number_of_columns - 2 {
                self.table_state.select_next_column();
            } else if self.selected_column_buffer
                < self.summary.cols_len() - self.number_of_columns + 1
            {
                self.selected_column_buffer += 1;
            } else {
                self.table_state.select_next_column();
            }
        }
    }

    fn previous_column(&mut self) {
        let table_selected_column = self.table_state.selected_column().unwrap_or(0);
        if self.selected_column_buffer + table_selected_column == 0 {
            self.select_last_column();
            self.previous_row();
        } else if table_selected_column > 1 {
            self.table_state.select_previous_column();
        } else if self.selected_column_buffer > 0 {
            self.selected_column_buffer = self.selected_column_buffer.saturating_sub(1);
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
                KeyCode::Home => self.select_first_column(),
                KeyCode::End => self.select_last_column(),
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
            .header_row(self.selected_column_buffer)
            .into_iter()
            .map(|name| Cell::from(name))
            .collect::<Row>()
            .style(header_style)
            .height(1);
        let rows = self
            .summary
            .get_rows(self.selected_column_buffer)
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
        let mut widths = vec![15];
        widths.extend(std::iter::repeat(9).take(self.number_of_columns));
        let t = Table::new(rows, widths)
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

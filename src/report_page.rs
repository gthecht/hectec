use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::Style,
    widgets::{Cell, Row, Table, TableState},
    Frame,
};

use crate::{table_design::add_design_to_table, transaction::TransactionsReport, TableColors};

pub struct ReportPage {
    report: TransactionsReport,
    selected_category: Option<String>,
    months_table_state: TableState,
    categories_table_state: TableState,
}

impl ReportPage {
    pub fn new() -> Self {
        ReportPage {
            report: TransactionsReport::new(&vec![]),
            selected_category: None,
            months_table_state: TableState::default().with_selected(0),
            categories_table_state: TableState::default(),
        }
    }

    pub fn reload(&mut self, report: TransactionsReport) {
        self.report = report;
    }

    fn update_selected(table_state: &mut TableState, i: Option<usize>) {
        table_state.select(i);
    }

    fn next_row(table_state: &mut TableState, rows_len: usize) {
        let i = table_state.selected().map_or(0, |i| {
            if i >= rows_len - 1 {
                rows_len - 1
            } else {
                i + 1
            }
        });
        Self::update_selected(table_state, Some(i));
    }

    fn previous_row(table_state: &mut TableState) {
        let i = table_state.selected().map_or(Some(0), |i| match i {
            0 => None,
            i => Some(i - 1),
        });
        Self::update_selected(table_state, i);
    }

    fn first_row(table_state: &mut TableState) {
        Self::update_selected(table_state, Some(0));
    }

    fn last_row(table_state: &mut TableState, rows_len: usize) {
        let i = match rows_len {
            0 => None,
            rl => Some(rl - 1),
        };
        Self::update_selected(table_state, i);
    }

    fn set_selected_category(&mut self) {
        let month_index = self.months_table_state.selected().unwrap_or(0);
        let category_index = self.categories_table_state.selected();
        self.selected_category = category_index
            .map(|index| {
                self.report
                    .get_category_by_index_for_month_at_index(month_index, index)
            })
            .flatten();
    }

    fn set_category_index(&mut self) {
        if self.selected_category == None {
            return;
        }
        // else look for the category in the current month's categories and set the index to that or None
        let month_index = self.months_table_state.selected().unwrap_or(0);
        let month_categories = self.report.get_categories_for_month_by_index(month_index);
        let category_index = month_categories
            .iter()
            .position(|c| Some(c) == self.selected_category.as_ref());
        Self::update_selected(&mut self.categories_table_state, category_index);
    }

    pub fn handle_key_events(&mut self, key: KeyEvent) {
        if key.kind == KeyEventKind::Press {
            let number_of_months = self.report.rows_len();
            let number_of_categories = self
                .report
                .get_categories_for_month_by_index(self.months_table_state.selected().unwrap_or(0))
                .len();
            match key.code {
                KeyCode::Down => {
                    Self::next_row(&mut self.months_table_state, number_of_months);
                    self.set_category_index();
                }
                KeyCode::Up => {
                    Self::previous_row(&mut self.months_table_state);
                    self.set_category_index();
                }
                KeyCode::PageUp => {
                    Self::first_row(&mut self.months_table_state);
                    self.set_category_index();
                }
                KeyCode::PageDown => {
                    Self::last_row(&mut self.months_table_state, number_of_months);
                    self.set_category_index();
                }
                KeyCode::Right => {
                    Self::next_row(&mut self.categories_table_state, number_of_categories);
                    self.set_selected_category();
                }
                KeyCode::Left => {
                    Self::previous_row(&mut self.categories_table_state);
                    self.set_selected_category();
                }
                KeyCode::Home => {
                    Self::first_row(&mut self.categories_table_state);
                    self.set_selected_category();
                }
                KeyCode::End => {
                    Self::last_row(&mut self.categories_table_state, number_of_categories);
                    self.set_selected_category();
                }
                _ => {}
            }
        }
    }

    pub fn draw(&mut self, frame: &mut Frame, area: Rect, colors: &TableColors) {
        let layout = &Layout::horizontal([Constraint::Length(24), Constraint::Min(42)]);
        let rects = layout.split(area);

        self.render_months(frame, rects[0], colors);
        self.render_categories(frame, rects[1], colors);
    }

    fn render_months(&mut self, frame: &mut Frame, area: Rect, colors: &TableColors) {
        let date_width = 10;
        let amount_width = 12;
        let header = Row::new(vec![
            "Dates".to_string(),
            format!(
                "{}",
                self.selected_category.clone().unwrap_or("".to_string())
            ),
        ]);
        let rows = self
            .report
            .get_month_rows(&self.selected_category)
            .into_iter()
            .map(|(month, value)| vec![month, format!("\n{:02.2}", value)])
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
        let widths = vec![date_width, amount_width];
        let t = add_design_to_table(Table::new(rows, widths), header, colors);
        frame.render_stateful_widget(t, area, &mut self.months_table_state);
    }

    fn render_categories(&mut self, frame: &mut Frame, area: Rect, colors: &TableColors) {
        let header = Row::new(vec!["Category", "Sum"]);
        let index = self.months_table_state.selected().unwrap_or(0);
        let rows = self
            .report
            .get_category_rows_for_month_by_index(index)
            .into_iter()
            .enumerate()
            .map(|(i, row)| {
                let color = match i % 2 {
                    0 => colors.normal_row_color,
                    _ => colors.alt_row_color,
                };

                row.into_iter()
                    .map(|v| Cell::from(v))
                    .collect::<Row>()
                    .style(Style::new().fg(colors.row_fg).bg(color))
                    .height(3)
            });
        let amount_width = 10;
        let category_width = area.as_size().width.max(amount_width + 4) - amount_width - 2;
        let widths = vec![category_width, amount_width];
        let t = add_design_to_table(Table::new(rows, widths), header, colors);
        frame.render_stateful_widget(t, area, &mut self.categories_table_state);
    }
}

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Text},
    widgets::{
        Bar, BarChart, BarGroup, Block, Cell, HighlightSpacing, Paragraph, Row, Table, TableState,
    },
    Frame,
};

use crate::{transaction::TransactionsReport, TableColors};

fn add_design_to_table<'a>(table: Table<'a>, header: Row<'a>, colors: &TableColors) -> Table<'a> {
    let header_style = Style::default().fg(colors.header_fg).bg(colors.header_bg);
    let selected_row_style = Style::default()
        .add_modifier(Modifier::REVERSED)
        .fg(colors.selected_row_style_fg);
    let selected_col_style = Style::default().fg(colors.selected_column_style_fg);
    let selected_cell_style = Style::default()
        .add_modifier(Modifier::REVERSED)
        .fg(colors.selected_cell_style_fg);
    let bar = " █ ";
    table
        .header(header.style(header_style).height(2))
        .row_highlight_style(selected_row_style)
        .column_highlight_style(selected_col_style)
        .cell_highlight_style(selected_cell_style)
        .highlight_symbol(Text::from(vec!["".into(), bar.into(), "".into()]))
        .bg(colors.buffer_bg)
        .highlight_spacing(HighlightSpacing::Always)
}

pub struct ReportPage {
    report: TransactionsReport,
    months_table_state: TableState,
    categories_table_state: TableState,
}

impl ReportPage {
    pub fn new() -> Self {
        ReportPage {
            report: TransactionsReport::new(&vec![]),
            months_table_state: TableState::default().with_selected(0),
            categories_table_state: TableState::default().with_selected(0),
        }
    }

    pub fn reload(&mut self, report: TransactionsReport) {
        self.report = report;
    }

    fn update_selected(table_state: &mut TableState, i: usize) {
        table_state.select(Some(i));
    }

    fn next_row(table_state: &mut TableState, rows_len: usize) {
        let i = match table_state.selected() {
            Some(i) => {
                if i >= rows_len - 1 {
                    rows_len - 1
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        Self::update_selected(table_state, i);
    }

    fn previous_row(table_state: &mut TableState) {
        let i = match table_state.selected() {
            Some(i) => {
                if i == 0 {
                    0
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        Self::update_selected(table_state, i);
    }

    fn first_row(table_state: &mut TableState) {
        let i = 0;
        Self::update_selected(table_state, i);
    }

    fn last_row(table_state: &mut TableState, rows_len: usize) {
        let i = rows_len - 1;
        Self::update_selected(table_state, i);
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
                    Self::first_row(&mut self.categories_table_state);
                }
                KeyCode::Up => {
                    Self::previous_row(&mut self.months_table_state);
                    Self::first_row(&mut self.categories_table_state);
                }
                KeyCode::PageUp => {
                    Self::first_row(&mut self.months_table_state);
                    Self::first_row(&mut self.categories_table_state);
                }
                KeyCode::PageDown => {
                    Self::last_row(&mut self.months_table_state, number_of_months);
                    Self::first_row(&mut self.categories_table_state);
                }
                KeyCode::Right => {
                    Self::next_row(&mut self.categories_table_state, number_of_categories)
                }
                KeyCode::Left => Self::previous_row(&mut self.categories_table_state),
                KeyCode::Home => Self::first_row(&mut self.categories_table_state),
                KeyCode::End => {
                    Self::last_row(&mut self.categories_table_state, number_of_categories)
                }
                _ => {}
            }
        }
    }

    pub fn draw(&mut self, frame: &mut Frame, area: Rect, colors: &TableColors) {
        let layout = &Layout::horizontal([
            Constraint::Length(15),
            Constraint::Min(30),
            Constraint::Length(50),
        ]);
        let rects = layout.split(area);

        self.render_months(frame, rects[0], colors);
        self.render_categories(frame, rects[2], colors);
        self.render_barchart(frame, rects[1], colors);
    }

    fn render_months(&mut self, frame: &mut Frame, area: Rect, colors: &TableColors) {
        let header = Row::new(vec!["\nDates"]);
        let rows = self
            .report
            .get_months()
            .into_iter()
            .map(|name| vec![name])
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
        let widths = vec![15];
        let t = add_design_to_table(Table::new(rows, widths), header, colors);
        frame.render_stateful_widget(t, area, &mut self.months_table_state);
    }

    fn render_categories(&mut self, frame: &mut Frame, area: Rect, colors: &TableColors) {
        let header = Row::new(vec!["\nCategory", "\nSum"]);
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
        let widths = vec![25, 10];
        let t = add_design_to_table(Table::new(rows, widths), header, colors);
        frame.render_stateful_widget(t, area, &mut self.categories_table_state);
    }

    fn render_barchart(&self, frame: &mut Frame, area: Rect, colors: &TableColors) {
        let [header_rect, space, bar_rect] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Min(3),
        ])
        .areas(area);
        let header = Paragraph::new(vec![Line::from(""), Line::from("bar graph - replace me")])
            .bg(colors.header_bg);
        frame.render_widget(header, header_rect);

        frame.render_widget(Block::new().bg(colors.normal_row_color), space);

        let temperatures: Vec<u8> = (0..24).map(|i| 50 - 2 * i).collect();
        let bars: Vec<Bar> = temperatures
            .into_iter()
            .map(|value| Self::horizontal_bar(value, colors))
            .collect();
        let barchart = BarChart::default()
            .data(BarGroup::default().bars(&bars))
            .bg(colors.normal_row_color)
            .bar_width(1)
            .bar_gap(2)
            .direction(ratatui::layout::Direction::Horizontal);
        frame.render_widget(barchart, bar_rect);
    }

    fn horizontal_bar(length: u8, colors: &TableColors) -> Bar {
        Bar::default()
            .value(u64::from(length))
            .style(colors.selected_column_style_fg)
            .value_style(colors.row_fg)
    }
}

use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Constraint, Layout, Position, Rect},
    style::{palette::tailwind, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Cell, Paragraph, Row, ScrollbarState, Table, TableState},
    Frame,
};

use crate::{
    table_design::add_design_to_table,
    transaction::{MonthInYear, TransactionField, TransactionsTable},
    TableColors,
};

enum ShouldAddNewRow {
    Yes,
    No,
}

pub struct InputPage {
    table_state: TableState,
    scroll_state: ScrollbarState,
    character_index: usize,
    pub transactions_table: TransactionsTable,
    input: String,
    error_msg: String,
}

impl InputPage {
    pub fn new(transactions_table: TransactionsTable) -> Self {
        Self {
            table_state: TableState::default().with_selected(0),
            scroll_state: ScrollbarState::new(0),
            character_index: 1,
            error_msg: "".to_string(),
            transactions_table,
            input: "".to_string(),
        }
    }

    pub fn initialize_table(&mut self) -> Result<()> {
        self.transactions_table.load()?;
        self.last_row();
        self.next_column();
        Ok(())
    }

    fn update_editing_text(&mut self) {
        if let Some((row, column)) = self.table_state.selected_cell() {
            if let Some(editing_text) = self.transactions_table.get_cell_text(row, column) {
                self.input = editing_text.clone();
                self.error_msg = "".to_string();
                self.character_index = self.input.chars().count();
                self.transactions_table
                    .update_recommended_input(row, column, &self.input);
            }
        }
    }

    fn update_selected(&mut self, i: usize) {
        self.table_state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i * 4); // each row is of height 4
        self.update_editing_text();
    }

    fn delete_transaction(&mut self) {
        if let Some(i) = self.table_state.selected() {
            self.transactions_table.delete_transaction(i);
        }
    }

    fn next_row(&mut self, add_new_row_if_end: ShouldAddNewRow) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= self.transactions_table.len() - 1 {
                    match add_new_row_if_end {
                        ShouldAddNewRow::Yes => {
                            self.transactions_table.new_transaction();
                            self.update_selected(self.transactions_table.len() - 1);
                        }
                        ShouldAddNewRow::No => {}
                    }
                    self.transactions_table.len() - 1
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
        let i = self.transactions_table.len() - 1;
        self.update_selected(i);
    }

    fn select_first_column(&mut self) {
        self.table_state.select_column(Some(0));
    }

    fn next_column(&mut self) {
        if self.table_state.selected_column() == Some(TransactionField::widths().len() - 1) {
            self.select_first_column();
            self.next_row(ShouldAddNewRow::No);
        } else {
            self.table_state.select_next_column();
        }
        self.update_editing_text();
    }

    fn previous_column(&mut self) {
        self.table_state.select_previous_column();
        self.update_editing_text();
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    fn move_cursor_to_end(&mut self) {
        let cursor_moved_to_end = self.input.len();
        self.character_index = self.clamp_cursor(cursor_moved_to_end);
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_home(&mut self) {
        let cursor_moved_home = 0;
        self.character_index = self.clamp_cursor(cursor_moved_home);
    }

    fn editing_text_byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    fn update_recommendation(&mut self) {
        if let Some((row, column)) = self.table_state.selected_cell() {
            self.transactions_table
                .update_recommended_input(row, column, &self.input);
        }
    }

    fn enter_char(&mut self, ch: char) {
        let index = self.editing_text_byte_index();
        self.input.insert(index, ch);
        self.move_cursor_right();
        self.update_recommendation();
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.character_index != 0;
        if is_not_cursor_leftmost {
            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            let after_char_to_delete = self.input.chars().skip(current_index);

            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
            self.update_recommendation();
        }
    }

    fn delete_char_forward(&mut self) {
        let current_character_index = self.character_index;
        self.move_cursor_right();
        if self.character_index > current_character_index {
            self.delete_char();
        } else {
            self.transactions_table.clear_recommended_input();
        }
    }

    fn commit_input(&mut self) -> Result<(), String> {
        if let Some((row, column)) = self.table_state.selected_cell() {
            self.transactions_table
                .update_transaction(row, column, &self.input)?;
        }
        Ok(())
    }

    pub fn handle_key_events(&mut self, key: KeyEvent) {
        if key.kind == KeyEventKind::Press {
            let ctrl_pressed = key.modifiers.contains(KeyModifiers::CONTROL);
            match key.code {
                KeyCode::Enter => match self.commit_input() {
                    Ok(()) => {
                        self.select_first_column();
                        self.next_row(ShouldAddNewRow::Yes);
                    }
                    Err(error) => self.error_msg = error.to_string(),
                },
                KeyCode::Tab => match self.commit_input() {
                    Ok(()) => self.next_column(),
                    Err(error) => self.error_msg = error.to_string(),
                },
                KeyCode::BackTab => match self.commit_input() {
                    Ok(()) => self.previous_column(),
                    Err(error) => self.error_msg = error.to_string(),
                },
                KeyCode::Down => self.next_row(ShouldAddNewRow::No),
                KeyCode::Up => self.previous_row(),
                KeyCode::PageUp => self.first_row(),
                KeyCode::PageDown => self.last_row(),
                KeyCode::Char('d') if ctrl_pressed => self.delete_transaction(),
                KeyCode::Backspace => self.delete_char(),
                KeyCode::Delete => self.delete_char_forward(),
                KeyCode::Left => self.move_cursor_left(),
                KeyCode::Right => self.move_cursor_right(),
                KeyCode::End => self.move_cursor_to_end(),
                KeyCode::Home => self.move_cursor_home(),
                KeyCode::Char(char_to_insert) => self.enter_char(char_to_insert),
                _ => {}
            }
        }
    }

    pub fn draw(&mut self, frame: &mut Frame, area: Rect, colors: &TableColors) {
        let vertical = &Layout::vertical([Constraint::Min(5), Constraint::Length(3)]);
        let rects = vertical.split(area);

        self.render_table(frame, rects[0], colors, true, (None, None));
        self.render_edit_bar(frame, rects[1], colors);
        let cursor_y = rects[1].as_position().y + 1;
        frame.set_cursor_position(Position::new(self.character_index as u16 + 1, cursor_y))
    }

    pub fn render_table(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        colors: &TableColors,
        highlight_selected: bool,
        filter: (Option<String>, Option<&MonthInYear>),
    ) {
        let header_style = Style::default().fg(colors.header_fg).bg(colors.header_bg);

        let header = TransactionField::names()
            .into_iter()
            .map(|name| Cell::from(name))
            .collect::<Row>()
            .style(header_style)
            .height(1);
        let rows = self
            .transactions_table
            .iter()
            .filter(|transaction| {
                filter
                    .0
                    .as_ref()
                    .map_or(true, |category| transaction.category == *category)
                    && filter.1.as_ref().map_or(true, |month_in_year| {
                        transaction.date.year == month_in_year.0
                            && transaction.date.month == month_in_year.1
                    })
            })
            .enumerate()
            .map(|(i, transaction)| {
                let color = match i % 2 {
                    0 => colors.normal_row_color,
                    _ => colors.alt_row_color,
                };
                let row = transaction.generate_row();
                row.style(Style::new().fg(colors.row_fg).bg(color))
                    .height(3)
            });
        let t = add_design_to_table(
            Table::new(rows, TransactionField::widths()),
            header,
            colors,
            highlight_selected,
        );
        frame.render_stateful_widget(t, area, &mut self.table_state);
    }

    fn render_edit_bar(&self, frame: &mut Frame, area: Rect, colors: &TableColors) {
        let edit_text = Line::from(vec![
            Span::from(&self.input),
            Span::from(&self.error_msg).fg(tailwind::ROSE.c600),
            Span::from(self.transactions_table.get_recommended_input(&self.input))
                .fg(tailwind::SLATE.c600),
        ]);
        let edit_bar = Paragraph::new(edit_text)
            .style(Style::new().fg(colors.row_fg).bg(colors.buffer_bg))
            .block(
                Block::bordered()
                    .border_type(BorderType::Double)
                    .border_style(Style::new().fg(colors.border_color)),
            );
        frame.render_widget(edit_bar, area);
    }
}

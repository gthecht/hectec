use std::fs;

mod transaction;
use crate::transaction::{Column, Transaction};
use color_eyre::Result;
use crossterm::event::{KeyEvent, KeyModifiers};
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Margin, Position, Rect},
    style::{self, Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{
        Block, BorderType, Cell, HighlightSpacing, Paragraph, Row, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Table, TableState,
    },
    DefaultTerminal, Frame,
};
use style::palette::tailwind;

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let file_path = "transactions.json";
    let app_result = App::new(file_path.to_string()).run(terminal);
    ratatui::restore();
    app_result
}

const PALETTES: [tailwind::Palette; 4] = [
    tailwind::BLUE,
    tailwind::EMERALD,
    tailwind::RED,
    tailwind::INDIGO,
];

const INFO_TEXT: [&str; 2] = [
    "(ESC) quit | (↑) move up | (↓ | ENTER) move down | (SHIFT+TAB) move left | (TAB) move right | PgUp go to first | PgDn go to last",
    "(CTRL+S) sort by selected column | (CTRL+N) new transaction | (CTRL+D) delete selected | (CTRL+C) change color",
];

struct TableColors {
    buffer_bg: Color,
    header_bg: Color,
    header_fg: Color,
    row_fg: Color,
    selected_row_style_fg: Color,
    selected_column_style_fg: Color,
    selected_cell_style_fg: Color,
    normal_row_color: Color,
    alt_row_color: Color,
    footer_border_color: Color,
}

impl TableColors {
    const fn new(color: &tailwind::Palette) -> Self {
        Self {
            buffer_bg: tailwind::SLATE.c950,
            header_fg: tailwind::SLATE.c200,
            row_fg: tailwind::SLATE.c200,
            normal_row_color: tailwind::SLATE.c950,
            alt_row_color: tailwind::SLATE.c900,
            header_bg: color.c900,
            selected_row_style_fg: color.c400,
            selected_column_style_fg: color.c400,
            selected_cell_style_fg: color.c600,
            footer_border_color: color.c400,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum SortOrder {
    Ascending,
    Descending,
}

impl std::ops::Not for SortOrder {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            SortOrder::Ascending => SortOrder::Descending,
            SortOrder::Descending => SortOrder::Ascending,
        }
    }
}

enum ShouldAddNewRow {
    Yes,
    No,
}

struct App {
    colors: TableColors,
    color_index: usize,
    table_state: TableState,
    scroll_state: ScrollbarState,
    sort_state: (usize, SortOrder),
    character_index: usize,
    columns: Vec<Column>,
    transactions: Vec<Transaction>,
    file_path: String,
    input: String,
    error_msg: String,
}

impl App {
    fn new(file_path: String) -> Self {
        let columns: Vec<Column> = vec![
            Column::new("Date", 11),
            Column::new("Amount", 10),
            Column::new("Details", 100),
            Column::new("Category", 15),
            Column::new("Method", 11),
            Column::new("Currency", 9),
        ];
        Self {
            colors: TableColors::new(&PALETTES[0]),
            color_index: 0,
            table_state: TableState::default().with_selected(0),
            scroll_state: ScrollbarState::new(0),
            sort_state: (0, SortOrder::Ascending),
            character_index: 1,
            input: "".to_string(),
            error_msg: "".to_string(),
            transactions: Vec::new(),
            file_path,
            columns,
        }
    }

    fn update_editing_text(&mut self) {
        if let Some((row, column)) = self.table_state.selected_cell() {
            if let Some(selected_transaction) = self.transactions.get(row) {
                let transaction_row = selected_transaction.generate_row_text();
                if let Some(editing_text) = transaction_row.get(column) {
                    self.input = editing_text.clone();
                    self.error_msg = "".to_string();
                    self.character_index = self.input.chars().count();
                }
            }
        }
    }

    fn update_selected(&mut self, i: usize) {
        self.table_state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i * 4); // each row is of height 4
        self.update_editing_text();
    }

    fn new_transaction(&mut self) {
        let last_transaction = self.transactions.last().unwrap();
        let new_transaction = Transaction::new(last_transaction.date);
        self.transactions.push(new_transaction);
        self.update_selected(self.transactions.len() - 1);
    }

    fn delete_transaction(&mut self) {
        match self.table_state.selected() {
            Some(i) => {
                if i > self.transactions.len() - 1 {
                } else {
                    self.transactions.remove(i);
                }
            }
            None => {}
        }
    }

    fn next_row(&mut self, add_new_row_if_end: ShouldAddNewRow) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= self.transactions.len() - 1 {
                    match add_new_row_if_end {
                        ShouldAddNewRow::Yes => self.new_transaction(),
                        ShouldAddNewRow::No => {}
                    }
                    self.transactions.len() - 1
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
        let i = self.transactions.len() - 1;
        self.update_selected(i);
    }

    fn select_first_column(&mut self) {
        self.table_state.select_column(Some(0));
    }

    fn next_column(&mut self) {
        if self.table_state.selected_column() == Some(self.columns.len() - 1) {
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

    fn next_color(&mut self) {
        self.color_index = (self.color_index + 1) % PALETTES.len();
        self.colors = TableColors::new(&PALETTES[self.color_index]);
    }

    fn sort_by_column(&mut self) {
        if let Some(column_index) = self.table_state.selected_column() {
            if self.sort_state.0 == column_index {
                self.sort_state.1 = !self.sort_state.1;
            } else {
                self.sort_state.1 = SortOrder::Ascending;
            }
            self.sort_state.0 = column_index;
            match self.columns.get(column_index) {
                Some(selected_column) => {
                    self.transactions.sort_by(|a, b| match self.sort_state.1 {
                        SortOrder::Ascending => Transaction::sort(a, b, selected_column),
                        SortOrder::Descending => Transaction::sort(b, a, selected_column),
                    });
                    self.update_editing_text();
                }
                None => {}
            }
        }
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

    fn enter_char(&mut self, ch: char) {
        let index = self.editing_text_byte_index();
        self.input.insert(index, ch);
        self.move_cursor_right();
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
        }
    }

    fn delete_char_forward(&mut self) {
        let current_character_index = self.character_index;
        self.move_cursor_right();
        if self.character_index > current_character_index {
            self.delete_char();
        }
    }

    fn commit_input(&mut self) -> Result<(), &str> {
        if let Some((row, column_index)) = self.table_state.selected_cell() {
            if let Some(transaction) = self.transactions.get_mut(row) {
                if let Some(column) = self.columns.get(column_index) {
                    transaction.mutate_field(column, &self.input)?
                }
            }
        }
        Ok(())
    }

    fn save_transactions(&mut self) -> Result<()> {
        self.transactions
            .sort_by(|a, b| Transaction::sort(a, b, &self.columns[0]));
        fs::write(
            &self.file_path,
            serde_json::to_string_pretty(&self.transactions)?,
        )?;
        Ok(())
    }

    fn handle_edit_events(&mut self, key: KeyEvent) -> Option<()> {
        if key.kind == KeyEventKind::Press {
            let ctrl_pressed = key.modifiers.contains(KeyModifiers::CONTROL);
            match key.code {
                KeyCode::Esc => return Some(()),
                KeyCode::Enter => match self.commit_input() {
                    Ok(()) => {
                        self.select_first_column();
                        self.next_row(ShouldAddNewRow::Yes);
                        // self.new_transaction();
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

                KeyCode::Char('c') if ctrl_pressed => self.next_color(),
                KeyCode::Char('s') if ctrl_pressed => self.sort_by_column(),
                KeyCode::Char('n') if ctrl_pressed => self.new_transaction(),
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
        None
    }

    fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        let file_string = fs::read_to_string(&self.file_path)?;
        self.transactions = serde_json::from_str(&file_string)?;
        self.transactions
            .sort_by(|a, b| Transaction::sort(a, b, &self.columns[0]));
        self.last_row();
        self.next_column();
        loop {
            terminal.draw(|frame| self.draw(frame))?;
            if let Event::Key(key) = event::read()? {
                if let Some(_) = self.handle_edit_events(key) {
                    self.save_transactions()?;
                    return Ok(());
                }
            }
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        let vertical = &Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(4),
        ]);
        let rects = vertical.split(frame.area());

        self.render_edit_bar(frame, rects[0]);
        self.render_table(frame, rects[1]);
        self.render_scrollbar(frame, rects[1]);
        self.render_footer(frame, rects[2]);
        frame.set_cursor_position(Position::new(self.character_index as u16 + 1, 1))
    }

    fn render_table(&mut self, frame: &mut Frame, area: Rect) {
        let header_style = Style::default()
            .fg(self.colors.header_fg)
            .bg(self.colors.header_bg);
        let selected_row_style = Style::default()
            .add_modifier(Modifier::REVERSED)
            .fg(self.colors.selected_row_style_fg);
        let selected_col_style = Style::default().fg(self.colors.selected_column_style_fg);
        let selected_cell_style = Style::default()
            .add_modifier(Modifier::REVERSED)
            .fg(self.colors.selected_cell_style_fg);

        let header = self
            .columns
            .iter()
            .map(|col| Cell::from(col.name().to_string()))
            .collect::<Row>()
            .style(header_style)
            .height(1);
        let rows = self
            .transactions
            .iter()
            .enumerate()
            .map(|(i, transaction)| {
                let color = match i % 2 {
                    0 => self.colors.normal_row_color,
                    _ => self.colors.alt_row_color,
                };
                let row = transaction.generate_row();
                row.style(Style::new().fg(self.colors.row_fg).bg(color))
                    .height(3)
            });
        let bar = " █ ";
        let t = Table::new(rows, self.columns.iter().map(|col| col.width))
            .header(header)
            .row_highlight_style(selected_row_style)
            .column_highlight_style(selected_col_style)
            .cell_highlight_style(selected_cell_style)
            .highlight_symbol(Text::from(vec!["".into(), bar.into(), "".into()]))
            .bg(self.colors.buffer_bg)
            .highlight_spacing(HighlightSpacing::Always);
        frame.render_stateful_widget(t, area, &mut self.table_state);
    }

    fn render_scrollbar(&mut self, frame: &mut Frame, area: Rect) {
        frame.render_stateful_widget(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None),
            area.inner(Margin {
                vertical: 1,
                horizontal: 1,
            }),
            &mut self.scroll_state,
        );
    }

    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let info_footer = Paragraph::new(Text::from_iter(INFO_TEXT))
            .style(
                Style::new()
                    .fg(self.colors.row_fg)
                    .bg(self.colors.buffer_bg),
            )
            .centered()
            .block(
                Block::bordered()
                    .border_type(BorderType::Double)
                    .border_style(Style::new().fg(self.colors.footer_border_color)),
            );
        frame.render_widget(info_footer, area);
    }

    fn render_edit_bar(&self, frame: &mut Frame, area: Rect) {
        let edit_text = Line::from(vec![
            Span::from(&self.input),
            Span::from(&self.error_msg).fg(tailwind::ROSE.c600),
        ]);
        let edit_bar = Paragraph::new(edit_text)
            .style(
                Style::new()
                    .fg(self.colors.row_fg)
                    .bg(self.colors.buffer_bg),
            )
            .block(
                Block::bordered()
                    .border_type(BorderType::Double)
                    .border_style(Style::new().fg(self.colors.footer_border_color)),
            );
        frame.render_widget(edit_bar, area);
    }
}

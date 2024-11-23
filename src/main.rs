use std::{cmp::Ordering, fs, str::FromStr};

use color_eyre::Result;
use crossterm::event::KeyModifiers;
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Margin, Position, Rect},
    style::{self, Color, Modifier, Style, Stylize},
    text::Text,
    widgets::{
        Block, BorderType, Cell, HighlightSpacing, Paragraph, Row, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Table, TableState,
    },
    DefaultTerminal, Frame,
};
use serde::{Deserialize, Serialize};
use style::palette::tailwind;
use time::{format_description, macros::format_description, Date, OffsetDateTime};

const PALETTES: [tailwind::Palette; 4] = [
    tailwind::BLUE,
    tailwind::EMERALD,
    tailwind::INDIGO,
    tailwind::RED,
];
const INFO_TEXT: [&str; 2] = [
    "(q) quit | (↑) move up | (↓) move down | (←) move left | (→) move right | HOME go to first | END go to last",
    "(s) sort by selected column | (Shift + →) next color | (Shift + ←) previous color",
];

const ITEM_HEIGHT: usize = 4;

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let file_path = "transactions.json";
    let file_string = fs::read_to_string(file_path)?;
    let transactions: Vec<Transaction> = serde_json::from_str(&file_string)?;
    let app_result = App::new(transactions).run(terminal);
    ratatui::restore();
    app_result
}

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
            header_bg: color.c900,
            header_fg: tailwind::SLATE.c200,
            row_fg: tailwind::SLATE.c200,
            selected_row_style_fg: color.c400,
            selected_column_style_fg: color.c400,
            selected_cell_style_fg: color.c600,
            normal_row_color: tailwind::SLATE.c950,
            alt_row_color: tailwind::SLATE.c900,
            footer_border_color: color.c400,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Transaction {
    #[serde(with = "time::serde::iso8601")]
    date: OffsetDateTime,
    amount: f64,
    currency: String,
    details: String,
    category: String,
    method: String,
}

struct Column {
    name: String,
    pub width: u16,
}

impl Column {
    pub fn new(name: &str, width: u16) -> Self {
        Self {
            name: name.to_string(),
            width,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Transaction {
    pub fn sort(a: &Transaction, b: &Transaction, column: &Column) -> Ordering {
        match column.name() {
            "Date" => a.date.cmp(&b.date),
            "Amount" => a.amount.partial_cmp(&b.amount).expect("amount not defined"),
            "Details" => a.details.cmp(&b.details),
            "Category" => a.category.cmp(&b.category),
            "Method" => a.method.cmp(&b.method),
            "Currency" => a.currency.cmp(&b.currency),
            &_ => a.date.cmp(&b.date), //warn("column not recognized")
        }
    }

    fn try_parse_date(&self, input: &str) -> Result<OffsetDateTime> {
        let format = format_description!(
            "[year]-[month]-[day] [hour]:[minute]:[second] [offset_hour \
                 sign:mandatory]"
        );
        Ok(OffsetDateTime::parse(
            &format!("{} 00:08:00 +02", input),
            &format,
        )?)
    }

    pub fn mutate_field(&mut self, column: &Column, input: &str) -> Result<(), String> {
        match column.name() {
            "Date" => match self.try_parse_date(input) {
                Ok(date) => self.date = date,
                Err(_) => return Err(format!("failed to parse {} as date", input)),
            },
            "Amount" => match f64::from_str(input) {
                Ok(num) => self.amount = num,
                Err(_) => return Err(format!("failed to parse {} as number", input)),
            },
            "Details" => self.details = input.to_string(),
            "Category" => self.category = input.to_string(),
            "Method" => self.method = input.to_string(),
            "Currency" => self.currency = input.to_string(),
            &_ => todo!(), //warn("column not recognized")
        }
        Ok(())
    }

    pub fn generate_row_text(&self) -> [String; 6] {
        [
            self.date
                .format(&format_description::parse("[year]-[month]-[day]").unwrap())
                .unwrap(),
            format!("{}", self.amount),
            self.details.clone(),
            self.category.clone(),
            self.method.clone(),
            self.currency.clone(),
        ]
    }

    pub fn generate_row(&self) -> Row {
        let cells: Vec<Cell> = self
            .generate_row_text()
            .iter()
            .map(|text| Cell::from(Text::from(format!("\n{}\n", text))))
            .collect();
        Row::new(cells)
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

enum InputMode {
    View,
    Edit,
    Quit,
}

struct App {
    colors: TableColors,
    color_index: usize,
    table_state: TableState,
    scroll_state: ScrollbarState,
    sort_state: (usize, SortOrder),
    input_mode: InputMode,
    character_index: usize,
    columns: Vec<Column>,
    items: Vec<Transaction>,
    input: String,
}

impl App {
    fn new(mut items: Vec<Transaction>) -> Self {
        let columns: Vec<Column> = vec![
            Column::new("Date", 11),
            Column::new("Amount", 10),
            Column::new("Details", 100),
            Column::new("Category", 15),
            Column::new("Method", 11),
            Column::new("Currency", 9),
        ];
        items.sort_by(|a, b| Transaction::sort(b, a, &columns[0]));
        Self {
            colors: TableColors::new(&PALETTES[0]),
            color_index: 0,
            table_state: TableState::default().with_selected(0),
            scroll_state: ScrollbarState::new((items.len() - 1) * ITEM_HEIGHT),
            sort_state: (0, SortOrder::Descending),
            input_mode: InputMode::View,
            character_index: 1,
            input: "".to_string(),
            columns,
            items,
        }
    }

    fn update_editing_text(&mut self) {
        if let Some((row, column)) = self.table_state.selected_cell() {
            if let Some(selected_item) = self.items.get(row) {
                let item_row = selected_item.generate_row_text();
                if let Some(editing_text) = item_row.get(column) {
                    self.input = editing_text.clone();
                    self.character_index = self.input.chars().count();
                }
            }
        }
    }

    fn update_selected(&mut self, i: usize) {
        self.table_state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
        self.update_editing_text();
    }

    pub fn next_row(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.update_selected(i);
    }

    pub fn previous_row(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.update_selected(i);
    }

    pub fn first_row(&mut self) {
        let i = 0;
        self.update_selected(i);
    }

    pub fn last_row(&mut self) {
        let i = self.items.len() - 1;
        self.update_selected(i);
    }

    pub fn next_column(&mut self) {
        self.table_state.select_next_column();
        self.update_editing_text();
    }

    pub fn previous_column(&mut self) {
        self.table_state.select_previous_column();
        self.update_editing_text();
    }

    pub fn next_color(&mut self) {
        self.color_index = (self.color_index + 1) % PALETTES.len();
    }

    pub fn previous_color(&mut self) {
        let count = PALETTES.len();
        self.color_index = (self.color_index + count - 1) % count;
    }

    pub fn set_colors(&mut self) {
        self.colors = TableColors::new(&PALETTES[self.color_index]);
    }

    pub fn sort_by_column(&mut self) {
        if let Some(column_index) = self.table_state.selected_column() {
            if self.sort_state.0 == column_index {
                self.sort_state.1 = !self.sort_state.1;
            } else {
                self.sort_state.1 = SortOrder::Descending;
            }
            self.sort_state.0 = column_index;
            match self.columns.get(column_index) {
                Some(selected_column) => self.items.sort_by(|a, b| match self.sort_state.1 {
                    SortOrder::Ascending => Transaction::sort(a, b, selected_column),
                    SortOrder::Descending => Transaction::sort(b, a, selected_column),
                }),
                None => {}
            }
        }
    }

    fn handle_view_events(&mut self) -> Result<()> {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                let shift_pressed = key.modifiers.contains(KeyModifiers::SHIFT);
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => self.input_mode = InputMode::Quit,
                    KeyCode::Char('j') | KeyCode::Down => self.next_row(),
                    KeyCode::Char('k') | KeyCode::Up => self.previous_row(),
                    KeyCode::Char('l') | KeyCode::Right if shift_pressed => self.next_color(),
                    KeyCode::Char('h') | KeyCode::Left if shift_pressed => {
                        self.previous_color();
                    }
                    KeyCode::Char('l') | KeyCode::Right | KeyCode::Tab => self.next_column(),
                    KeyCode::Char('h') | KeyCode::Left | KeyCode::BackTab => self.previous_column(),
                    KeyCode::Home => self.first_row(),
                    KeyCode::End => self.last_row(),
                    KeyCode::Char('s') => self.sort_by_column(),
                    KeyCode::Char('e') | KeyCode::Enter => self.input_mode = InputMode::Edit,
                    _ => {}
                }
            }
        }
        Ok(())
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

    fn commit_input(&mut self) -> Result<(), String> {
        if let Some((row, column_index)) = self.table_state.selected_cell() {
            if let Some(item) = self.items.get_mut(row) {
                if let Some(column) = self.columns.get(column_index) {
                    item.mutate_field(column, &self.input)?
                }
            }
        }
        Ok(())
    }

    fn handle_edit_events(&mut self) -> Result<()> {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Esc => self.input_mode = InputMode::View,
                    KeyCode::Enter => {
                        match self.commit_input() {
                            Ok(()) => self.next_row(),
                            Err(error) => self.input = error,
                        }
                        self.input_mode = InputMode::View;
                    }
                    KeyCode::Down => self.next_row(),
                    KeyCode::Up => self.previous_row(),
                    KeyCode::Tab => self.next_column(),
                    KeyCode::BackTab => self.previous_column(),
                    KeyCode::Char(char_to_insert) => self.enter_char(char_to_insert),
                    KeyCode::Backspace => self.delete_char(),
                    KeyCode::Delete => self.delete_char_forward(),
                    KeyCode::Left => self.move_cursor_left(),
                    KeyCode::Right => self.move_cursor_right(),
                    KeyCode::End => self.move_cursor_to_end(),
                    KeyCode::Home => self.move_cursor_home(),
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;

            match self.input_mode {
                InputMode::Quit => return Ok(()),
                InputMode::View => {
                    self.handle_view_events()
                        .expect("could not handle event correctly");
                }
                InputMode::Edit => {
                    self.handle_edit_events()
                        .expect("could not handle event correctly");
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

        self.set_colors();

        self.render_edit_bar(frame, rects[0]);
        self.render_table(frame, rects[1]);
        self.render_scrollbar(frame, rects[1]);
        self.render_footer(frame, rects[2]);
        match self.input_mode {
            InputMode::Edit => {
                frame.set_cursor_position(Position::new(self.character_index as u16 + 1, 1))
            }
            _ => {}
        }
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
        let rows = self.items.iter().enumerate().map(|(i, item)| {
            let color = match i % 2 {
                0 => self.colors.normal_row_color,
                _ => self.colors.alt_row_color,
            };
            let row = item.generate_row();
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
        let edit_text = Text::from(self.input.clone());
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

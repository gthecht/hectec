mod input_page;
mod instructions;
mod logger;
mod transaction;
use std::env;
use std::path::PathBuf;

use crate::input_page::InputPage;
use crate::instructions::Instructions;
use crate::logger::initialize_logging;
use crate::transaction::TransactionsTable;
use color_eyre::Result;
use crossterm::event::{KeyEvent, KeyModifiers};
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Margin, Rect},
    style::{self, Color},
    widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState},
    DefaultTerminal, Frame,
};
use style::palette::tailwind;

fn main() -> Result<()> {
    color_eyre::install()?;
    initialize_logging()?;
    let args: Vec<String> = env::args().collect();
    let terminal = ratatui::init();
    let default_path = PathBuf::from("transactions.csv");
    let file_path = args
        .get(1)
        .map_or(default_path, |input| PathBuf::from(input));
    let app_result = App::new(file_path).run(terminal);
    ratatui::restore();
    app_result
}

const PALETTES: [tailwind::Palette; 4] = [
    tailwind::BLUE,
    tailwind::EMERALD,
    tailwind::RED,
    tailwind::INDIGO,
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
    border_color: Color,
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
            border_color: color.c400,
        }
    }
}

struct App {
    colors: TableColors,
    color_index: usize,
    scroll_state: ScrollbarState,
    input_page: InputPage,
    instructions: Instructions,
}

impl App {
    fn new(file_path: PathBuf) -> Self {
        let transactions_table = TransactionsTable::new(file_path);
        let input_page = InputPage::new(transactions_table);
        Self {
            colors: TableColors::new(&PALETTES[0]),
            color_index: 0,
            scroll_state: ScrollbarState::new(0),
            input_page,
            instructions: Instructions::oneline(),
        }
    }

    fn next_color(&mut self) {
        self.color_index = (self.color_index + 1) % PALETTES.len();
        self.colors = TableColors::new(&PALETTES[self.color_index]);
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Option<()> {
        if key.kind == KeyEventKind::Press {
            let ctrl_pressed = key.modifiers.contains(KeyModifiers::CONTROL);
            match key.code {
                KeyCode::Esc => return Some(()),
                KeyCode::Char('c') if ctrl_pressed => self.next_color(),
                KeyCode::Char('h') if ctrl_pressed => self.instructions.toggle(),
                _ => self.input_page.handle_key_events(key),
            }
        }
        None
    }

    fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.input_page.initialize_table()?;
        loop {
            terminal.draw(|frame| self.draw(frame))?;
            if let Event::Key(key) = event::read()? {
                if let Some(_) = self.handle_key_events(key) {
                    self.input_page.transactions_table.save_transactions()?;
                    return Ok(());
                }
            }
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        let vertical = &Layout::vertical([
            Constraint::Length(self.instructions.get_height()),
            Constraint::Min(8),
        ]);
        let rects = vertical.split(frame.area());

        self.instructions.draw(frame, rects[0], &self.colors);
        self.input_page.draw(frame, rects[1], &self.colors);
        self.render_scrollbar(frame, rects[1]);
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
}

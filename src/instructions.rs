use ratatui::{
    layout::Rect,
    style::Style,
    text::Text,
    widgets::{Block, BorderType, Paragraph},
    Frame,
};

use crate::TableColors;

const ONELINE_INSTRUCTIONS: [&str; 1] =
    ["ESC => save & quit | CTRL+H => open help instructions | CTRL+C => change color"];

const FULL_INSTRUCTIONS_HEIGHT: u16 = 10;
const FULL_INSTRUECTIONS: [&str; FULL_INSTRUCTIONS_HEIGHT as usize] = [
    "ESC => save & quit",
    "CTRL+H => close help instructions",
    "CTRL+C => change color",
    "↑ => one line up | ↓/ENTER => one line down",
    "ENTER at last line => create new transaction",
    "SHIFT+TAB => previous-column",
    "TAB => next-column & insert recommended text",
    "PgUp => go to first row | PgDn => go to last row",
    "CTRL+D => delete selected row",
    "DEL at end of text => remove recommended text",
];

enum State {
    Full,
    Oneline,
}

pub struct Instructions {
    instructions: Vec<&'static str>,
    height: u16,
    state: State,
}

impl Instructions {
    pub fn oneline() -> Self {
        Self {
            instructions: Vec::from(ONELINE_INSTRUCTIONS),
            height: 1,
            state: State::Oneline,
        }
    }

    pub fn full() -> Self {
        Self {
            instructions: Vec::from(FULL_INSTRUECTIONS),
            height: FULL_INSTRUCTIONS_HEIGHT,
            state: State::Full,
        }
    }

    pub fn toggle(&mut self) {
        match self.state {
            State::Full => *self = Self::oneline(),
            State::Oneline => *self = Self::full(),
        };
    }

    pub fn get_height(&self) -> u16 {
        self.height + 2
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect, colors: &TableColors) {
        let instructions = Paragraph::new(Text::from_iter(self.instructions.clone()))
            .style(Style::new().fg(colors.row_fg).bg(colors.buffer_bg))
            .block(
                Block::bordered()
                    .border_type(BorderType::Double)
                    .border_style(Style::new().fg(colors.border_color)),
            );
        frame.render_widget(instructions, area);
    }
}

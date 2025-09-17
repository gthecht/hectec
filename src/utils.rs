use crossterm::event::{KeyEvent, KeyModifiers};

pub fn ctrl_is_pressed(key: &KeyEvent) -> bool {
    key.modifiers.contains(KeyModifiers::CONTROL)
}

use ratatui::{
    style::{Modifier, Style, Stylize},
    text::Text,
    widgets::{Block, BorderType, HighlightSpacing, Row, Table},
};

use crate::TableColors;

pub fn add_design_to_table<'a>(
    table: Table<'a>,
    header: Row<'a>,
    colors: &TableColors,
    highlight_selected: bool,
) -> Table<'a> {
    let header_style = Style::default().fg(colors.header_fg).bg(colors.header_bg);
    let selected_row_style = Style::default()
        .add_modifier(Modifier::REVERSED)
        .fg(colors.selected_row_style_fg);
    let selected_col_style = Style::default().fg(colors.selected_column_style_fg);
    let selected_cell_style = Style::default()
        .add_modifier(Modifier::REVERSED)
        .fg(colors.selected_cell_style_fg);
    let bar = " █ ";
    let formatted_table = table
        .header(header.style(header_style))
        .bg(colors.buffer_bg)
        .highlight_spacing(HighlightSpacing::Always)
        .block(
            Block::bordered()
                .border_type(BorderType::Double)
                .border_style(Style::new().fg(colors.border_color)),
        );
    if highlight_selected {
        return formatted_table
            .row_highlight_style(selected_row_style)
            .column_highlight_style(selected_col_style)
            .cell_highlight_style(selected_cell_style)
            .highlight_symbol(Text::from(vec!["".into(), bar.into(), "".into()]));
    } else {
        return formatted_table;
    }
}

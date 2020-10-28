use std::io;

use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, List, Text};
use tui::Terminal;
use tui::backend::Backend;

use crate::{Explorer, Player};

pub fn draw<B: Backend>(terminal: &mut Terminal<B>, explorer: &mut Explorer, player: &mut Player, search: &str) -> io::Result<()> {
    terminal.draw(|mut f| {
        let constraints = if search.is_empty() {
            [Constraint::Percentage(100), Constraint::Percentage(0)]
        } else {
            [Constraint::Percentage(98), Constraint::Percentage(2)]
        };
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints.as_ref())
            .split(f.size());
        let main = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(chunks[0]);

        let dir_strings = explorer.selected_dir().entry_strings();
        let current_dir = explorer
            .current_dir_name()
            .unwrap_or_else(|| "Music".to_string());
        let block = list(&current_dir, &dir_strings);
        f.render_stateful_widget(block, main[0], explorer.list_state());

        if !player.playing().is_empty() {
            player.list_state.select(Some(player.index()));
        }
        let playing_strings: Vec<String> = player
            .playing()
            .iter()
            .map(|p| p.file_name().unwrap().to_os_string().into_string().unwrap())
            .collect();
        let volume = format!("Volume: {:.0}", player.volume() * 100f32);
        let block = list(&volume, &playing_strings);
        f.render_stateful_widget(block, main[1], &mut player.list_state);

        let search_bar = Block::default()
            .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
            .title(&search);
        f.render_widget(search_bar, chunks[1]);
    })
}

fn list<'a>(
    title: &'a str,
    items: &'a Vec<String>,
) -> List<'a, impl Iterator<Item = Text<'a>> + 'a> {
    let block = Block::default().title(title).borders(Borders::ALL);
    let style = Style::default().bg(Color::Green).modifier(Modifier::BOLD);
    List::new(items.iter().map(Text::raw))
        .block(block)
        .highlight_style(style)
}

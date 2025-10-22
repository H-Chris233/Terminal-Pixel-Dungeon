//! 设置菜单状态（基础占位实现）

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::input::{map_to_ui_action, KeyBindings, UIAction};
use crate::input::{NavDirection, NavigationState};
use crate::states::common::{GameState, GameStateID, StateContext, StateTransition};

#[derive(Debug)]
pub struct SettingsMenuState {
    options: Vec<&'static str>,
    nav: NavigationState,
    bindings: KeyBindings,
}

impl SettingsMenuState {
    pub fn new() -> Self {
        let options = vec!["Audio: On", "Brightness: 100%", "Auto-Save: 5m", "Back"];
        Self {
            nav: NavigationState::new(options.len()),
            options,
            bindings: KeyBindings::default(),
        }
    }
}

impl GameState for SettingsMenuState {
    fn id(&self) -> GameStateID { GameStateID::Settings }

    fn handle_input(&mut self, context: &mut StateContext, event: &crossterm::event::Event) -> bool {
        if let Some(action) = map_to_ui_action(event, &self.bindings) {
            match action {
                UIAction::NavigateUp => { self.nav.navigate(NavDirection::Up); }
                UIAction::NavigateDown => { self.nav.navigate(NavDirection::Down); }
                UIAction::Confirm => {
                    if self.nav.current() == self.options.len() - 1 { // Back
                        context.request_pop();
                    }
                }
                UIAction::Cancel => { context.request_pop(); }
                _ => {}
            }
            return true;
        }
        false
    }

    fn render(&mut self, context: &mut StateContext) -> anyhow::Result<()> {
        context.terminal.draw(|f| {
            let area = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(5),
                ])
                .split(f.area())[1];

        let items: Vec<Line> = self.options.iter().enumerate().map(|(i, text)| {
            let focused = i == self.nav.current();
            Line::from(Span::styled(
                if focused { format!("> {} <", text) } else { format!("  {}  ", text) },
                Style::default().fg(if focused { Color::Yellow } else { Color::White })
            ))
        }).collect();

        let block = Paragraph::new(items)
            .alignment(Alignment::Center)
            .block(Block::default().title("Settings").borders(Borders::ALL));
        f.render_widget(block, area);
        })?;
        Ok(())
    }

    fn enter_transition(&self) -> Option<StateTransition> { Some(StateTransition::fade(0.3)) }
}

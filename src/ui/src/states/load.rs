//! 读档菜单状态：列出 saves/ 目录下的存档并选择加载

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::input::{map_to_ui_action, KeyBindings, UIAction};
use crate::input::navigation::{NavDirection, NavigationState};
use crate::states::common::{GameState, GameStateID, StateContext, StateTransition};
use save::{SaveSystem, SaveData};

#[derive(Debug)]
pub struct LoadMenuState {
    entries: Vec<String>,
    nav: NavigationState,
    bindings: KeyBindings,
}

impl LoadMenuState {
    pub fn new() -> Self {
        // 读取 saves/ 目录下的文件名（占位：显示最多10个槽位）
        let save_system = SaveSystem::new("saves", 10).ok();
        let entries = if let Some(sys) = save_system {
            (0..sys.max_slots())
                .map(|slot| {
                    if sys.has_save(slot) {
                        format!("Slot {}: Saved", slot)
                    } else {
                        format!("Slot {}: Empty", slot)
                    }
                })
                .collect::<Vec<_>>()
        } else {
            vec!["Failed to init save system".to_string()]
        };
        let nav = NavigationState::new(entries.len());
        Self { entries, nav, bindings: KeyBindings::default() }
    }
}

impl GameState for LoadMenuState {
    fn id(&self) -> GameStateID { GameStateID::LoadMenu }

    fn handle_input(&mut self, context: &mut StateContext, event: &crossterm::event::Event) -> bool {
        if let Some(action) = map_to_ui_action(event, &self.bindings) {
            match action {
                UIAction::NavigateUp => { self.nav.navigate(NavDirection::Up); }
                UIAction::NavigateDown => { self.nav.navigate(NavDirection::Down); }
                UIAction::Confirm => {
                    let slot = self.nav.current();
                    if let Ok(sys) = SaveSystem::new("saves", 10) {
                        if sys.has_save(slot) {
                            if let Ok(data) = sys.load_game(slot) {
                                // 由 StateContext 传递预构造的 Gameplay 状态
                                if let Ok(state) = crate::states::game::GameplayState::from_save(data) {
                                    context.set_pending_state(Box::new(state));
                                    return Some(GameStateID::Gameplay).is_some();
                                }
                            }
                        }
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

        let items: Vec<Line> = self.entries.iter().enumerate().map(|(i, text)| {
            let focused = i == self.nav.current();
            Line::from(Span::styled(
                if focused { format!("> {} <", text) } else { format!("  {}  ", text) },
                Style::default().fg(if focused { Color::Yellow } else { Color::White })
            ))
        }).collect();

        let block = Paragraph::new(items)
            .alignment(Alignment::Center)
            .block(Block::default().title("Load Game").borders(Borders::ALL));
        f.render_widget(block, area);
        })?;
        Ok(())
    }

    fn enter_transition(&self) -> Option<StateTransition> { Some(StateTransition::fade(0.3)) }
}

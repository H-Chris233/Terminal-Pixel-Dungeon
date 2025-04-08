//! 菜单系统状态
//!
//! 实现像素地牢风格的菜单系统：
//! - 8-bit像素风格UI渲染
//! - 键盘导航系统
//! - 状态过渡动画

use super::*;
use crossterm::event::KeyCode;
use tui::widgets::Widget;
use tui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
};

use crate::ui::states::common::GameState;
use crate::ui::states::common::GameStateID;
use crate::ui::states::common::StateContext;
use crate::ui::states::common::StateTransition;

/// 主菜单状态
#[derive(Debug)]
pub struct MainMenuState {
    selected_index: usize,
    options: Vec<&'static str>,
    version: &'static str,
    blink_timer: f32,
}

impl MainMenuState {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            options: vec!["New Game", "Load Game", "Settings", "Quit"],
            version: "v0.0.1",
            blink_timer: 0.0,
        }
    }

    /// 渲染标题艺术字（像素风格）
    fn render_title(&self) -> Spans {
        Spans::from(vec![
            Span::styled("PIXEL ", Style::default().fg(Color::Red)),
            Span::styled("DUNGEON", Style::default().fg(Color::White)),
        ])
    }

    /// 渲染菜单选项
    fn render_options(&self, selected: bool, idx: usize) -> Span {
        let option = self.options[idx];
        if idx == self.selected_index && selected {
            Span::styled(
                format!("> {} <", option),
                Style::default().fg(Color::Yellow),
            )
        } else {
            Span::styled(option, Style::default().fg(Color::Gray))
        }
    }
}

impl GameState for MainMenuState {
    fn id(&self) -> GameStateID {
        GameStateID::MainMenu
    }

    fn handle_input(
        &mut self,
        context: &mut StateContext,
        event: &crossterm::event::Event,
    ) -> bool {
        if let Some(key) = context.input.match_key(event) {
            match key {
                KeyCode::Up => {
                    self.selected_index = self.selected_index.saturating_sub(1);
                    true
                }
                KeyCode::Down => {
                    self.selected_index = (self.selected_index + 1).min(self.options.len() - 1);
                    true
                }
                KeyCode::Enter => {
                    match self.selected_index {
                        0 => {
                            //context.audio.play_sound("confirm.ogg");
                            Some(GameStateID::Gameplay)
                        }
                        1 => {
                            //context.audio.play_sound("error.ogg"); // 暂未实现
                            None
                        }
                        3 => {
                            context.request_quit();
                            None
                        }
                        _ => None,
                    }
                }
                _ => false,
            }
        } else {
            false
        }
    }

    fn update(&mut self, context: &mut StateContext, delta_time: f32) -> Option<GameStateID> {
        // 光标闪烁动画
        self.blink_timer += delta_time;
        if self.blink_timer > 1.0 {
            self.blink_timer = 0.0;
        }
        None
    }

    fn render(&mut self, context: &mut StateContext) -> anyhow::Result<()> {
        context.terminal.draw(|f| {
            let size = f.size();

            // 主标题
            let title_block = Paragraph::new(self.render_title())
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::NONE));

            // 版本信息
            let version_block = Paragraph::new(self.version)
                .alignment(Alignment::Right)
                .style(Style::default().fg(Color::DarkGray));

            // 菜单选项（带闪烁效果）
            let show_cursor = self.blink_timer < 0.5;
            let menu_items: Vec<Spans> = self
                .options
                .iter()
                .enumerate()
                .map(|(i, _)| Spans::from(self.render_options(show_cursor, i)))
                .collect();

            let menu_block = Paragraph::new(menu_items)
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::NONE));

            // 布局
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(5), // 标题
                    Constraint::Length(8), // 菜单
                    Constraint::Min(1),    // 空白
                ])
                .split(size);

            f.render_widget(title_block, chunks[0]);
            f.render_widget(menu_block, chunks[1]);
            f.render_widget(version_block, size);
        })?;

        Ok(())
    }

    fn enter_transition(&self) -> Option<StateTransition> {
        Some(StateTransition::fade(0.8))
    }
}

/// 暂停菜单状态
#[derive(Debug)]
pub struct PauseMenuState {
    selected_index: usize,
    options: Vec<&'static str>,
}

impl PauseMenuState {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            options: vec!["Continue", "Save Game", "Main Menu", "Quit"],
        }
    }
}

impl GameState for PauseMenuState {
    fn id(&self) -> GameStateID {
        GameStateID::PauseMenu
    }

    fn handle_input(
        &mut self,
        context: &mut StateContext,
        event: &crossterm::event::Event,
    ) -> bool {
        if let Some(key) = context.input.match_key(event) {
            match key {
                KeyCode::Esc => {
                    //context.audio.play_sound("resume.ogg");
                    Some(GameStateID::Gameplay)
                }
                KeyCode::Up => {
                    self.selected_index = self.selected_index.saturating_sub(1);
                    true
                }
                KeyCode::Down => {
                    self.selected_index = (self.selected_index + 1).min(self.options.len() - 1);
                    true
                }
                KeyCode::Enter => match self.selected_index {
                    0 => Some(GameStateID::Gameplay),
                    2 => Some(GameStateID::MainMenu),
                    3 => {
                        context.request_quit();
                        None
                    }
                    _ => None,
                },
                _ => false,
            }
        } else {
            false
        }
    }

    fn render(&mut self, context: &mut StateContext) -> anyhow::Result<()> {
        context.terminal.draw(|f| {
            let size = f.size();
            let area = centered_rect(50, 50, size);

            let block = Block::default()
                .title("PAUSED")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red));

            let menu: Vec<Spans> = self
                .options
                .iter()
                .enumerate()
                .map(|(i, text)| {
                    let style = if i == self.selected_index {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    Spans::from(Span::styled(
                        if i == self.selected_index {
                            format!("> {} <", text)
                        } else {
                            text.to_string()
                        },
                        style,
                    ))
                })
                .collect();

            let paragraph = Paragraph::new(menu)
                .block(block)
                .alignment(Alignment::Center);

            f.render_widget(paragraph, area);
        })?;

        Ok(())
    }

    fn block_lower_states(&self) -> bool {
        true
    }
}

/// 游戏结束菜单
#[derive(Debug)]
pub struct GameOverState {
    score: u32,
    cause_of_death: String,
    blink_timer: f32,
}

impl GameOverState {
    pub fn new(score: u32, cause: &str) -> Self {
        Self {
            score,
            cause_of_death: cause.to_string(),
            blink_timer: 0.0,
        }
    }
}

impl GameState for GameOverState {
    fn id(&self) -> GameStateID {
        GameStateID::GameOver
    }

    fn handle_input(
        &mut self,
        context: &mut StateContext,
        event: &crossterm::event::Event,
    ) -> bool {
        context.input.match_key(event, KeyCode::Enter)
    }

    fn update(&mut self, _: &mut StateContext, delta_time: f32) -> Option<GameStateID> {
        self.blink_timer += delta_time;
        None
    }

    fn render(&mut self, context: &mut StateContext) -> anyhow::Result<()> {
        context.terminal.draw(|f| {
            let size = f.size();
            let area = centered_rect(60, 40, size);

            let show_prompt = self.blink_timer % 1.0 < 0.5;
            let text = vec![
                Spans::from(Span::styled("YOU DIED!", Style::default().fg(Color::Red))),
                Spans::from(Span::raw("")),
                Spans::from(Span::styled(
                    &self.cause_of_death,
                    Style::default().fg(Color::White),
                )),
                Spans::from(Span::raw("")),
                Spans::from(Span::styled(
                    format!("Score: {}", self.score),
                    Style::default().fg(Color::Yellow),
                )),
                Spans::from(Span::raw("")),
                Spans::from(if show_prompt {
                    Span::styled("Press ENTER to continue", Style::default().fg(Color::Gray))
                } else {
                    Span::raw("")
                }),
            ];

            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red));

            Paragraph::new(text)
                .block(block)
                .alignment(Alignment::Center)
                .render(f, area);
        })?;

        Ok(())
    }

    fn enter_transition(&self) -> Option<StateTransition> {
        Some(StateTransition::fade(1.0))
    }
}

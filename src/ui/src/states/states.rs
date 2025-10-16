//! 状态堆栈管理器
//!
//! 实现像素地牢风格的状态管理系统：
//! - 分层状态管理（支持暂停菜单覆盖游戏界面）
//! - 平滑过渡动画（淡入淡出/滑动）
//! - 智能输入路由

use super::{
    common::{GameState, GameStateID, StateContext, StateTransition},
    menu::{GameOverState, MainMenuState, PauseMenuState},
    settings::SettingsMenuState,
    load::LoadMenuState,
    game::GameplayState,
};
use crate::input::InputSystem;
use crate::render::render::RenderSystem;
use crate::terminal::TerminalController;
use anyhow::Result;
use crossterm::event::Event;
use std::time::Instant;

/// 状态堆栈管理器
pub struct StateStack {
    states: Vec<Box<dyn GameState>>,
    context: StateContext,
    transition: Option<(StateTransition, GameStateID)>,
    transition_start: Option<Instant>,
    transition_target: Option<GameStateID>,
}

impl StateStack {
    /// 创建新状态堆栈
    pub fn new(
        terminal: TerminalController,
        input: InputSystem,
        render: RenderSystem,
        initial_state: GameStateID,
    ) -> Self {
        let mut stack = Self {
            states: Vec::with_capacity(3),
            context: StateContext::new(terminal, input, render),
            transition: None,
            transition_start: None,
            transition_target: None,
        };
        stack.push_state(initial_state);
        stack
    }

    /// 压入新状态（带过渡动画）
    pub fn push_state(&mut self, state_id: GameStateID) {
        if let Some(top_state) = self.states.last() {
            if let Some(transition) = top_state.enter_transition() {
                self.start_transition(transition, state_id);
                return;
            }
        }
        self.instant_push(state_id);
    }

    /// 立即压入状态（无过渡）
    fn instant_push(&mut self, state_id: GameStateID) {
        // 优先使用 pending_state（由菜单预构造）
        if let Some(mut state) = self.context.pending_state.take() {
            state.on_enter(&mut self.context);
            self.states.push(state);
            return;
        }

        let mut state: Box<dyn GameState> = match state_id {
            GameStateID::MainMenu => Box::new(MainMenuState::new()),
            GameStateID::PauseMenu => Box::new(PauseMenuState::new()),
            GameStateID::GameOver => Box::new(GameOverState::new(0, "Unknown cause")),
            GameStateID::Settings => Box::new(SettingsMenuState::new()),
            GameStateID::LoadMenu => Box::new(LoadMenuState::new()),
            GameStateID::Gameplay => Box::new(GameplayState::new(1).expect("Init gameplay")),
            _ => Box::new(MainMenuState::new()),
        };
        state.on_enter(&mut self.context);
        self.states.push(state);
    }

    /// 弹出当前状态（带过渡动画）
    pub fn pop_state(&mut self) {
        if let Some(top_state) = self.states.last() {
            if let Some(transition) = top_state.exit_transition() {
                self.start_transition(transition, GameStateID::MainMenu);
                return;
            }
        }
        self.instant_pop();
    }

    /// 立即弹出状态（无过渡）
    fn instant_pop(&mut self) {
        if let Some(mut state) = self.states.pop() {
            state.on_exit(&mut self.context);
        }
        // 清理弹出请求标记
        self.context.clear_pop_request();
    }

    /// 开始状态过渡
    fn start_transition(&mut self, transition: StateTransition, target: GameStateID) {
        self.transition = Some((transition, target));
        self.transition_start = Some(Instant::now());
        self.transition_target = Some(target);
    }

    /// 完成当前过渡
    fn complete_transition(&mut self) {
        if let Some(target) = self.transition_target.take() {
            if self.states.iter().any(|s| s.id() == target) {
                while let Some(state) = self.states.last() {
                    if state.id() == target { break; }
                    self.instant_pop();
                }
            } else {
                self.instant_push(target);
            }
        }
        self.transition = None;
    }

    /// 处理输入事件（从栈顶向栈底传递）
    pub fn handle_input(&mut self, event: &Event) {
        if self.transition.is_some() {
            return;
        }

        for state in self.states.iter_mut().rev() {
            if state.handle_input(&mut self.context, event) {
                break;
            }
            if state.block_lower_states() {
                break;
            }
        }

        if let Some(id) = self.context.take_push_request() {
            self.push_state(id);
        }
        if self.context.pop_request {
            self.pop_state();
        }
    }

    /// 更新状态逻辑
    pub fn update(&mut self, delta_time: f32) -> bool {
        if let Some((transition, _)) = &mut self.transition {
            self.context.transition_progress = transition.progress();
            if transition.update(delta_time) {
                self.complete_transition();
            }
            return !self.context.should_quit;
        }

        for state in self.states.iter_mut().rev() {
            if let Some(new_state) = state.update(&mut self.context, delta_time) {
                self.push_state(new_state);
                break;
            }
            if state.block_lower_states() {
                break;
            }
        }
        !self.context.should_quit
    }

    /// 渲染所有可见状态（从栈底向栈顶渲染）
    pub fn render(&mut self) -> Result<()> {
        let _transition_progress = self.transition.as_ref().map_or(0.0, |(t, _)| t.progress());
        let len = self.states.len();
        for (i, state) in self.states.iter_mut().enumerate() {
            if i == len - 1 || !state.block_lower_states() {
                state.render(&mut self.context)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_transitions() {
        let terminal = TerminalController::new().expect("Terminal init");
        let input = InputSystem::default();
        let render = RenderSystem::new();
        let mut stack = StateStack::new(terminal, input, render, GameStateID::MainMenu);

        stack.push_state(GameStateID::Gameplay);
        assert_eq!(stack.states.len(), 2);

        stack.pop_state();
        assert_eq!(stack.states.len(), 1);
    }
}

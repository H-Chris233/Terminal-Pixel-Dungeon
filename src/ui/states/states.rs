
//! 状态堆栈管理器
//!
//! 实现像素地牢风格的状态管理系统：
//! - 分层状态管理（支持暂停菜单覆盖游戏界面）
//! - 平滑过渡动画（淡入淡出/滑动）
//! - 智能输入路由

use super::{
    game::GameState,
    menu::{MainMenuState, PauseMenuState, GameOverState},
    common::{StateTransition, GameStateID, GameState},
};
use crate::{
    ui::{
        terminal::TerminalController,
        input::{InputSystem, KeyCode},
        render::RenderSystem,
    },
    util::math::lerp,
};
use anyhow::Result;
use crossterm::event::Event;
use std::time::{Duration, Instant};

/// 状态堆栈管理器
pub struct StateStack {
    states: Vec<Box<dyn GameState>>,
    context: StateContext,
    transition: Option<(StateTransition, GameStateID)>,
    transition_start: Option<Instant>,
    transition_target: Option<GameStateID>,
}

/// 状态共享上下文
pub struct StateContext {
    pub terminal: TerminalController,
    pub input: InputSystem,
    pub render: RenderSystem,
    pub audio: AudioSystem,
    pub should_quit: bool,
    pub transition_progress: f32,
}

impl StateContext {
    pub fn new(
        terminal: TerminalController,
        input: InputSystem,
        render: RenderSystem,
        audio: AudioSystem,
    ) -> Self {
        Self {
            terminal,
            input,
            render,
            audio,
            should_quit: false,
            transition_progress: 0.0,
        }
    }
}

impl StateStack {
    /// 创建新状态堆栈
    pub fn new(
        terminal: TerminalController,
        input: InputSystem,
        render: RenderSystem,
        audio: AudioSystem,
        initial_state: GameStateID,
    ) -> Self {
        let mut stack = Self {
            states: Vec::with_capacity(3), // 预分配3层状态
            context: StateContext::new(terminal, input, render, audio),
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
        let state: Box<dyn GameState> = match state_id {
            GameStateID::MainMenu => Box::new(MainMenuState::new()),
            GameStateID::Gameplay => Box::new(GameState::new(1)),
            GameStateID::PauseMenu => Box::new(PauseMenuState::new()),
            GameStateID::GameOver => Box::new(GameOverState::new(0, "Unknown cause")),
            _ => unimplemented!(),
        };
        state.on_enter(&mut self.context);
        self.states.push(state);
    }

    /// 弹出当前状态（带过渡动画）
    pub fn pop_state(&mut self) {
        if let Some(top_state) = self.states.last() {
            if let Some(transition) = top_state.exit_transition() {
                self.start_transition(transition, GameStateID::MainMenu); // 临时目标，实际在complete_transition处理
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
                // 如果目标状态已在堆栈中，则弹出到该状态
                while let Some(state) = self.states.last() {
                    if state.id() == target { break; }
                    self.instant_pop();
                }
            } else {
                // 否则压入新状态
                self.instant_push(target);
            }
        }
        self.transition = None;
    }

    /// 处理输入事件（从栈顶向栈底传递）
    pub fn handle_input(&mut self, event: &Event) {
        if self.transition.is_some() { return; }

        for state in self.states.iter_mut().rev() {
            if state.handle_input(&mut self.context, event) {
                break;
            }
            if state.pause_lower_states() {
                break;
            }
        }
    }

    /// 更新状态逻辑
    pub fn update(&mut self, delta_time: f32) -> bool {
        // 处理过渡动画
        if let Some((transition, _)) = &mut self.transition {
            self.context.transition_progress = transition.progress();
            if transition.update(delta_time) {
                self.complete_transition();
            }
            return !self.context.should_quit;
        }

        // 从栈顶开始更新（直到遇到暂停状态）
        for state in self.states.iter_mut().rev() {
            if let Some(new_state) = state.update(&mut self.context, delta_time) {
                self.push_state(new_state);
                break;
            }
            if state.pause_lower_states() {
                break;
            }
        }

        !self.context.should_quit
    }

    /// 渲染所有可见状态（从栈底向栈顶渲染）
    pub fn render(&mut self) -> Result<()> {
        // 计算过渡效果进度
        let transition_progress = self.transition.as_ref()
            .map_or(0.0, |(t, _)| t.progress());

        // 渲染非阻塞状态
        for (i, state) in self.states.iter_mut().enumerate() {
            if i == self.states.len() - 1 || !state.block_lower_states() {
                state.render(&mut self.context)?;
            }
        }

        // 渲染过渡效果
        if let Some((transition, _)) = &self.transition {
            match transition {
                StateTransition::Fade { .. } => {
                    self.context.render.render_fade(
                        &mut self.context.terminal,
                        transition_progress
                    )?;
                }
                StateTransition::Slide { direction, .. } => {
                    self.context.render.render_slide(
                        &mut self.context.terminal,
                        *direction,
                        transition_progress
                    )?;
                }
                _ => {}
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;

    #[test]
    fn test_state_transitions() {
        let terminal = TerminalController::new();
        let input = InputSystem::new();
        let render = RenderSystem::new();
        let audio = AudioSystem::new();
        let mut stack = StateStack::new(terminal, input, render, audio, GameStateID::MainMenu);
        
        // 测试状态压栈
        stack.push_state(GameStateID::Gameplay);
        assert_eq!(stack.states.len(), 2);
        assert_eq!(stack.states[1].id(), GameStateID::Gameplay);
        
        // 测试状态弹出
        stack.pop_state();
        assert_eq!(stack.states.len(), 1);
        assert_eq!(stack.states[0].id(), GameStateID::MainMenu);
    }
}

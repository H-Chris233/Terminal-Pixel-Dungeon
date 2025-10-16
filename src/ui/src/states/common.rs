//! 状态机核心组件
//!
//! 实现像素地牢风格的状态管理系统：
//! - 支持淡入淡出/滑动过渡动画
//! - 严格的状态生命周期控制
//! - 输入事件优先级路由

use crate::input::InputSystem;
use crate::render::render::RenderSystem;
use crate::terminal::TerminalController;

fn lerp(a: f32, b: f32, t: f32) -> f32 { a + (b - a) * t }
use anyhow::Result;
use crossterm::event::Event;

/// 滑动方向（用于过渡动画）
#[derive(Debug, Clone, Copy)]
pub enum SlideDirection {
    Left,
    Right,
    Up,
    Down,
}

/// 状态过渡效果（像素地牢经典风格）
#[derive(Debug)]
pub enum StateTransition {
    None, // 立即切换
    Fade {
        duration: f32, // 过渡时长（秒）
        progress: f32, // 当前进度 [0.0, 1.0]
    },
    Slide {
        direction: SlideDirection,
        duration: f32,
        progress: f32,
    },
}

impl StateTransition {
    /// 创建新的淡入淡出过渡
    pub fn fade(duration: f32) -> Self {
        Self::Fade { duration, progress: 0.0 }
    }

    /// 创建新的滑动过渡
    pub fn slide(direction: SlideDirection, duration: f32) -> Self {
        Self::Slide { direction, duration, progress: 0.0 }
    }

    /// 更新过渡动画进度
    pub fn update(&mut self, delta_time: f32) -> bool {
        match self {
            Self::Fade { duration, progress } => {
                *progress = (*progress + delta_time / *duration).min(1.0);
                *progress >= 1.0
            }
            Self::Slide { duration, progress, .. } => {
                *progress = (*progress + delta_time / *duration).min(1.0);
                *progress >= 1.0
            }
            Self::None => true,
        }
    }

    /// 获取当前过渡进度（用于渲染）
    pub fn progress(&self) -> f32 {
        match self {
            Self::Fade { progress, .. } => *progress,
            Self::Slide { progress, .. } => *progress,
            Self::None => 1.0,
        }
    }
}

/// 游戏状态标识符
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameStateID {
    MainMenu,
    Gameplay,
    PauseMenu,
    Inventory,
    Shop,
    GameOver,
    Victory,
    Settings,
    LoadMenu, // 新增：读档菜单
}

/// 基础状态特征
pub trait GameState: std::fmt::Debug {
    /// 获取状态唯一标识
    fn id(&self) -> GameStateID;

    /// 处理输入事件（返回是否消耗该事件）
    fn handle_input(&mut self, context: &mut StateContext, event: &Event) -> bool {
        let _ = context;
        let _ = event;
        false
    }

    /// 更新状态逻辑（返回需要切换到的目标状态）
    fn update(&mut self, context: &mut StateContext, delta_time: f32) -> Option<GameStateID> {
        let _ = context;
        let _ = delta_time;
        None
    }

    /// 渲染状态界面
    fn render(&mut self, context: &mut StateContext) -> Result<()>;

    /// 状态进入时的回调（适合初始化资源）
    fn on_enter(&mut self, context: &mut StateContext) {
        let _ = context;
    }

    /// 状态退出时的回调（适合清理资源）
    fn on_exit(&mut self, context: &mut StateContext) {
        let _ = context;
    }

    /// 是否应该暂停下层状态的渲染
    fn block_lower_states(&self) -> bool { true }

    /// 获取当前过渡动画（用于压入新状态时）
    fn enter_transition(&self) -> Option<StateTransition> { Some(StateTransition::fade(0.3)) }

    /// 获取退出过渡动画（用于弹出状态时）
    fn exit_transition(&self) -> Option<StateTransition> { Some(StateTransition::fade(0.2)) }
}

/// 状态共享上下文数据
pub struct StateContext {
    pub terminal: TerminalController,
    pub input: InputSystem,
    pub render: RenderSystem,
    pub should_quit: bool,
    pub transition_progress: f32, // 全局过渡进度
    pub pending_state: Option<Box<dyn GameState>>, // 待压入的预构造状态
    pub pop_request: bool, // 请求弹出顶部状态
    pub push_request: Option<GameStateID>, // 请求压入的目标状态ID
}

impl std::fmt::Debug for StateContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "StateContext {{ should_quit: {}, pop_request: {} }}", self.should_quit, self.pop_request)
    }
}

impl StateContext {
    pub fn new(terminal: TerminalController, input: InputSystem, render: RenderSystem) -> Self {
        Self {
            terminal,
            input,
            render,
            should_quit: false,
            transition_progress: 0.0,
            pending_state: None,
            pop_request: false,
            push_request: None,
        }
    }

    /// 处理退出游戏逻辑
    pub fn request_quit(&mut self) { self.should_quit = true; }

    /// 设置待压入的预构造状态
    pub fn set_pending_state(&mut self, state: Box<dyn GameState>) { self.pending_state = Some(state); }

    /// 请求压入指定状态
    pub fn request_push(&mut self, id: GameStateID) { self.push_request = Some(id); }
    pub fn take_push_request(&mut self) -> Option<GameStateID> { self.push_request.take() }

    /// 请求弹出当前状态
    pub fn request_pop(&mut self) { self.pop_request = true; }

    /// 清除弹出请求
    pub fn clear_pop_request(&mut self) { self.pop_request = false; }
}

/// 状态渲染辅助方法
pub mod render_util {
    use super::*;
    use ratatui::{layout::Rect, style::Color};

    /// 应用淡入淡出效果到区域
    pub fn apply_fade(area: &mut Rect, progress: f32) {
        let width = lerp(area.width as f32 * 0.8, area.width as f32, progress) as u16;
        let height = lerp(area.height as f32 * 0.8, area.height as f32, progress) as u16;
        area.x = area.x.saturating_add((area.width - width) / 2);
        area.y = area.y.saturating_add((area.height - height) / 2);
        area.width = width;
        area.height = height;
    }

    /// 应用滑动效果到区域
    pub fn apply_slide(area: &mut Rect, direction: SlideDirection, progress: f32) {
        match direction {
            SlideDirection::Left => {
                area.x = lerp(area.x as f32 - area.width as f32, area.x as f32, progress) as u16;
            }
            SlideDirection::Right => {
                area.x = lerp(area.x as f32 + area.width as f32, area.x as f32, progress) as u16;
            }
            SlideDirection::Up => {
                area.y = lerp(area.y as f32 - area.height as f32, area.y as f32, progress) as u16;
            }
            SlideDirection::Down => {
                area.y = lerp(area.y as f32 + area.height as f32, area.y as f32, progress) as u16;
            }
        }
    }

    /// 计算过渡颜色（用于淡入淡出效果）
    pub fn transition_color(base: Color, _progress: f32) -> Color { base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct MockState;
    impl GameState for MockState {
        fn id(&self) -> GameStateID { GameStateID::Gameplay }
        fn render(&mut self, _: &mut StateContext) -> Result<()> { Ok(()) }
    }

    #[test]
    fn test_fade_transition() {
        let mut fade = StateTransition::fade(1.0);
        assert!(!fade.update(0.5));
        assert_eq!(fade.progress(), 0.5);
        assert!(fade.update(0.5));
    }
}

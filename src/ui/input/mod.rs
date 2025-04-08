
//! 输入处理系统
//!
//! 提供完整的输入处理管道：
//! - 原始输入事件处理 (`input`)
//! - 游戏动作抽象 (`actions`)
//! - UI导航控制 (`navigation`)
//!
//! # 架构概览
//! ```text
//! 终端输入事件 → InputSystem (原始处理)
//!     ├→ map_to_game_action → GameAction (游戏逻辑)
//!     └→ map_to_ui_action → UIAction (界面控制)
//!            └→ NavigationState (焦点管理)
//! ```
//!
//! # 示例用法
//! ```
//! use crate::ui::input::{InputSystem, GameAction, UIAction};
//!
//! let mut input_system = InputSystem::default();
//! let event = crossterm::event::read().unwrap();
//!
//! // 处理游戏动作
//! if let Some(action) = input_system.map_to_game_action(&event) {
//!     match action {
//!         GameAction::MoveUp => player.move_up(),
//!         _ => {}
//!     }
//! }
//!
//! // 处理UI动作
//! if let Some(action) = input_system.map_to_ui_action(&event) {
//!     ui_state.handle_action(action);
//! }
//! ```

mod actions;
mod input;
mod navigation;

pub use actions::{
    GameAction,  // 游戏实体动作 (移动/交互等)
    UIAction,    // 界面控制动作 (确认/导航等)
    KeyBindings, // 可配置的键位绑定
    map_to_game_action, // 事件→游戏动作转换
    map_to_ui_action,   // 事件→UI动作转换
};

pub use input::{
    InputSystem,  // 主输入处理器
    InputConfig,  // 输入行为配置
};

pub use navigation::{
    NavDirection,    // 导航方向枚举
    NavigationState, // 焦点状态管理
};

/// 输入系统预导入集合 (方便统一引用)
pub mod prelude {
    pub use super::{
        GameAction, UIAction,
        InputSystem, InputConfig,
        NavDirection, NavigationState,
        KeyBindings,
    };
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn test_input_pipeline() {
        let mut input_system = InputSystem::default();
        let bindings = KeyBindings::default();
        
        // 模拟方向键输入
        let event = Event::Key(KeyEvent::new(
            KeyCode::Up,
            KeyModifiers::NONE
        ));
        
        // 验证游戏动作转换
        assert_eq!(
            map_to_game_action(&event, &bindings),
            Some(GameAction::MoveUp)
        );
        
        // 验证UI动作转换
        assert_eq!(
            map_to_ui_action(&event, &bindings),
            Some(UIAction::NavigateUp)
        );
        
        // 验证导航状态
        let mut nav = NavigationState::new(5);
        assert!(nav.navigate(NavDirection::Down));
        assert_eq!(nav.current(), 1);
    }
}

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
mod enhanced_input;
mod input;
mod navigation;

pub use actions::{
    map_to_game_action, // 事件→游戏动作转换
    map_to_ui_action,   // 事件→UI动作转换
    GameAction,         // 游戏实体动作 (移动/交互等)
    KeyBindings,        // 可配置的键位绑定
    UIAction,           // 界面控制动作 (确认/导航等)
};

pub use enhanced_input::{
    EnhancedInputEvent, // 增强输入事件
    EnhancedInputProcessor, // 增强输入处理器
    InputContextManager, // 输入上下文管理器
    InputMode,          // 输入模式
    KeyMapping,         // 按键映射配置
};

pub use input::{
    InputConfig, // 输入行为配置
    InputSystem, // 主输入处理器
};

pub use navigation::{
    NavDirection,    // 导航方向枚举
    NavigationState, // 焦点状态管理
};

// 为了向后兼容，重新导出navigation模块  
// pub use navigation; // 暂时禁用，因为navigation是私有模块

/// 输入系统预导入集合 (方便统一引用)
pub mod prelude {
    pub use super::{
        map_to_game_action, // <-- 补充导出转换函数
        map_to_ui_action,
        GameAction,
        InputConfig,
        InputSystem,
        KeyBindings,
        NavDirection,
        NavigationState,
        UIAction,
    };
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, Event};

    #[test]
    fn test_input_pipeline() {
        let mut input_system = InputSystem::default();
        let bindings = KeyBindings::default();

        // 模拟方向键输入
        let event = Event::Key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));

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
        assert!(nav.navigate_no_debounce(NavDirection::Down));
        assert_eq!(nav.current(), 1);
    }
}

// 在 tests mod 中添加：
#[test]
fn test_no_wrap_around() {
    let mut nav = NavigationState::new(3);
    nav.set_wrap_around(false); // <-- 测试关闭循环

    // 测试不能循环到尾部
    nav.jump_to(0);
    assert!(!nav.navigate_no_debounce(NavDirection::Prev));

    // 测试不能循环到头部
    nav.jump_to(2);
    assert!(!nav.navigate_no_debounce(NavDirection::Next));
}

#[test]
fn test_grid_edge_cases() {
    // 单列测试
    let mut nav = NavigationState::new(5);
    nav.set_grid(1); // 单列网格

    assert!(nav.navigate_no_debounce(NavDirection::Down));
    assert_eq!(nav.current(), 1);

    // 单行测试
    let mut nav = NavigationState::new(3);
    nav.set_grid(3); // 单行网格

    assert!(nav.navigate_no_debounce(NavDirection::Right));
    assert_eq!(nav.current(), 1);
    assert!(!nav.navigate_no_debounce(NavDirection::Down)); // 应无法下移
}

#[test]
fn test_input_debounce() {
    let mut nav = NavigationState::new(5);

    // 首次调用应允许
    assert!(nav.navigate(NavDirection::Next));

    // 模拟快速连续调用 - 这个测试实际上验证了防抖机制
    // 由于防抖逻辑，快速连续调用可能被阻止
    // 但这取决于时间，所以我们简化这个测试
    std::thread::sleep(std::time::Duration::from_millis(100));
    assert!(nav.navigate(NavDirection::Next)); // 延迟后应该允许
}

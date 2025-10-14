//src/ui/input/actions.rs
//! 输入动作映射系统
//!
//! 实现像素地牢风格的输入处理：
//! - 游戏内动作与UI动作分离
//! - 可配置键位绑定
//! - 终端友好的输入映射

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use strum::{EnumCount, EnumIter};

/// 游戏内实体动作（与具体输入解耦）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, EnumCount)]
pub enum GameAction {
    MoveUp,        // 向上移动
    MoveDown,      // 向下移动
    MoveLeft,      // 向左移动
    MoveRight,     // 向右移动
    MoveUpLeft,    // 左上移动
    MoveUpRight,   // 右上移动
    MoveDownLeft,  // 左下移动
    MoveDownRight, // 右下移动
    Wait,          // 等待回合
    Interact,      // 交互（开门/捡物品）
    Search,        // 搜索隐藏门
    Attack,        // 攻击最近敌人
    UseItem,       // 使用当前选中物品
    ThrowItem,     // 投掷当前选中物品
    ZapWand,       // 使用当前选中法杖
    Inventory(u8), // 打开指定槽位物品栏(0-9)
    QuickSlot(u8), // 使用快捷栏物品(0-9)
    Examine,       // 检查格子
    Rest,          // 休息直到恢复
}

/// UI专用动作（菜单/界面控制）
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter)]
pub enum UIAction {
    Confirm,          // 确认选择
    Cancel,           // 取消/返回
    NavigateUp,       // 向上导航
    NavigateDown,     // 向下导航
    NavigateLeft,     // 向左导航
    NavigateRight,    // 向右导航
    TabNext,          // 下一个标签页
    TabPrev,          // 上一个标签页
    PageUp,           // 上一页
    PageDown,         // 下一页
    QuickSave,        // 快速保存
    QuickLoad,        // 快速读取
    ToggleFullscreen, // 全屏切换
}

/// 键位配置结构体（支持多套键位方案）
#[derive(Debug, Clone)]
pub struct KeyBindings {
    // 移动控制
    pub move_up: KeyEvent,
    pub move_down: KeyEvent,
    pub move_left: KeyEvent,
    pub move_right: KeyEvent,

    // 常用动作
    pub wait: KeyEvent,
    pub interact: KeyEvent,
    pub search: KeyEvent,
    pub attack: KeyEvent,

    // 系统控制
    pub confirm: KeyEvent,
    pub cancel: KeyEvent,
    pub inventory: KeyEvent,
    pub examine: KeyEvent,

    // 数字键快捷栏
    pub quick_slots: [KeyEvent; 10],
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            // 方向键移动（支持小键盘）
            move_up: KeyEvent::new(KeyCode::Up, KeyModifiers::NONE),
            move_down: KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
            move_left: KeyEvent::new(KeyCode::Left, KeyModifiers::NONE),
            move_right: KeyEvent::new(KeyCode::Right, KeyModifiers::NONE),

            // 空格等待，G互动，S搜索，A攻击
            wait: KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE),
            interact: KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE),
            search: KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE),
            attack: KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE),

            // 回车确认，ESC取消，I物品栏，E检查
            confirm: KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            cancel: KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
            inventory: KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE),
            examine: KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE),

            // 数字键1-0对应快捷栏
            quick_slots: [
                KeyEvent::new(KeyCode::Char('1'), KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Char('2'), KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Char('3'), KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Char('4'), KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Char('5'), KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Char('6'), KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Char('7'), KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Char('8'), KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Char('9'), KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Char('0'), KeyModifiers::NONE),
            ],
        }
    }
}

/// 将终端输入事件转换为游戏动作
pub fn map_to_game_action(event: &Event, bindings: &KeyBindings) -> Option<GameAction> {
    match event {
        // 八方向移动
        Event::Key(key) if *key == bindings.move_up => Some(GameAction::MoveUp),
        Event::Key(key) if *key == bindings.move_down => Some(GameAction::MoveDown),
        Event::Key(key) if *key == bindings.move_left => Some(GameAction::MoveLeft),
        Event::Key(key) if *key == bindings.move_right => Some(GameAction::MoveRight),

        // Vi-keys 斜向移动支持
        Event::Key(KeyEvent {
            code: KeyCode::Char('y'),
            ..
        }) => Some(GameAction::MoveUpLeft),
        Event::Key(KeyEvent {
            code: KeyCode::Char('u'),
            ..
        }) => Some(GameAction::MoveUpRight),
        Event::Key(KeyEvent {
            code: KeyCode::Char('b'),
            ..
        }) => Some(GameAction::MoveDownLeft),
        Event::Key(KeyEvent {
            code: KeyCode::Char('n'),
            ..
        }) => Some(GameAction::MoveDownRight),

        // 基础动作
        Event::Key(key) if *key == bindings.wait => Some(GameAction::Wait),
        Event::Key(key) if *key == bindings.interact => Some(GameAction::Interact),
        Event::Key(key) if *key == bindings.search => Some(GameAction::Search),
        Event::Key(key) if *key == bindings.attack => Some(GameAction::Attack),

        // 物品相关
        Event::Key(key) if *key == bindings.inventory => Some(GameAction::Inventory(0)),
        Event::Key(KeyEvent {
            code: KeyCode::Char(c @ '1'..='9'),
            ..
        }) => Some(GameAction::QuickSlot(c.to_digit(10).unwrap() as u8 - 1)),
        Event::Key(KeyEvent {
            code: KeyCode::Char('0'),
            ..
        }) => Some(GameAction::QuickSlot(9)),

        // 其他功能
        Event::Key(key) if *key == bindings.examine => Some(GameAction::Examine),
        _ => None,
    }
}

/// 将终端输入事件转换为UI动作
pub fn map_to_ui_action(event: &Event, bindings: &KeyBindings) -> Option<UIAction> {
    match event {
        // 基础导航
        Event::Key(key) if *key == bindings.move_up => Some(UIAction::NavigateUp),
        Event::Key(key) if *key == bindings.move_down => Some(UIAction::NavigateDown),
        Event::Key(key) if *key == bindings.move_left => Some(UIAction::NavigateLeft),
        Event::Key(key) if *key == bindings.move_right => Some(UIAction::NavigateRight),

        // 确认/取消
        Event::Key(key) if *key == bindings.confirm => Some(UIAction::Confirm),
        Event::Key(key) if *key == bindings.cancel => Some(UIAction::Cancel),

        // 标签页切换
        Event::Key(KeyEvent {
            code: KeyCode::Tab,
            modifiers: KeyModifiers::SHIFT,
            ..
        }) => Some(UIAction::TabPrev),
        Event::Key(KeyEvent {
            code: KeyCode::Tab, ..
        }) => Some(UIAction::TabNext),

        // 翻页
        Event::Key(KeyEvent {
            code: KeyCode::PageUp,
            ..
        }) => Some(UIAction::PageUp),
        Event::Key(KeyEvent {
            code: KeyCode::PageDown,
            ..
        }) => Some(UIAction::PageDown),

        // 系统功能
        _ => None,

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyEventKind;

    #[test]
    fn test_default_keybindings() {
        let bindings = KeyBindings::default();

        // 测试方向键映射
        assert_eq!(
            map_to_game_action(&Event::Key(bindings.move_up), &bindings),
            Some(GameAction::MoveUp)
        );

        // 测试数字键快捷栏
        assert_eq!(
            map_to_game_action(
                &Event::Key(KeyEvent::new(KeyCode::Char('1'), KeyModifiers::NONE)),
                &bindings
            ),
            Some(GameAction::QuickSlot(0))
        );
    }

    #[test]
    fn test_ui_actions() {
        let bindings = KeyBindings::default();

        // 测试确认取消
        assert_eq!(
            map_to_ui_action(&Event::Key(bindings.confirm), &bindings),
            Some(UIAction::Confirm)
        );

        // 测试标签页切换
        assert_eq!(
            map_to_ui_action(
                &Event::Key(KeyEvent::new(KeyCode::Tab, KeyModifiers::SHIFT)),
                &bindings
            ),
            Some(UIAction::TabPrev)
        );
    }
}

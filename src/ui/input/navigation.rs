//! 终端UI导航系统
//!
//! 实现像素地牢风格的导航控制：
//! - 四方向/标签页导航
//! - 焦点循环与边界处理
//! - 键盘/手柄输入统一抽象

use super::actions::UIAction;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::{Duration, Instant};

/// 导航方向（兼容终端和手柄输入）
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NavDirection {
    Up,    // 上导航
    Down,  // 下导航
    Left,  // 左导航
    Right, // 右导航
    Next,  // 下一个标签页/项目
    Prev,  // 上一个标签页/项目
}

impl NavDirection {
    /// 从终端输入事件转换导航方向（参考像素地牢PC版键位）
    pub fn from_event(event: &Event) -> Option<Self> {
        match event {
            // 方向键
            Event::Key(KeyEvent {
                code: KeyCode::Up, ..
            }) => Some(Self::Up),
            Event::Key(KeyEvent {
                code: KeyCode::Down,
                ..
            }) => Some(Self::Down),
            Event::Key(KeyEvent {
                code: KeyCode::Left,
                ..
            }) => Some(Self::Left),
            Event::Key(KeyEvent {
                code: KeyCode::Right,
                ..
            }) => Some(Self::Right),

            // WASD移动（与游戏内移动一致）
            Event::Key(KeyEvent {
                code: KeyCode::Char('w'),
                modifiers: KeyModifiers::NONE,
                ..
            }) => Some(Self::Up),
            Event::Key(KeyEvent {
                code: KeyCode::Char('s'),
                modifiers: KeyModifiers::NONE,
                ..
            }) => Some(Self::Down),
            Event::Key(KeyEvent {
                code: KeyCode::Char('a'),
                modifiers: KeyModifiers::NONE,
                ..
            }) => Some(Self::Left),
            Event::Key(KeyEvent {
                code: KeyCode::Char('d'),
                modifiers: KeyModifiers::NONE,
                ..
            }) => Some(Self::Right),

            // 标签页切换
            Event::Key(KeyEvent {
                code: KeyCode::Tab,
                modifiers: KeyModifiers::SHIFT,
                ..
            }) => Some(Self::Prev),
            Event::Key(KeyEvent {
                code: KeyCode::Tab,
                modifiers: KeyModifiers::NONE,
                ..
            }) => Some(Self::Next),

            // 数字键快捷导航（如物品栏1-9）
            Event::Key(KeyEvent {
                code: KeyCode::Char(c @ '1'..='9'),
                ..
            }) => Some(if c < &'5' { Self::Prev } else { Self::Next }),
            _ => None,
        }
    }

    /// 转换为UI动作（用于状态机处理）
    pub fn to_action(self) -> UIAction {
        match self {
            Self::Up => UIAction::NavigateUp,
            Self::Down => UIAction::NavigateDown,
            Self::Left => UIAction::NavigateLeft,
            Self::Right => UIAction::NavigateRight,
            Self::Next => UIAction::TabNext,
            Self::Prev => UIAction::TabPrev,
        }
    }
}

/// UI导航状态机（带输入防抖）
#[derive(Debug, Clone)]
pub struct NavigationState {
    current_focus: usize,      // 当前聚焦项索引
    item_count: usize,         // 总项目数
    wrap_around: bool,         // 是否允许循环导航
    grid_width: Option<usize>, // 网格布局列数（用于物品栏等）
    last_input_time: Instant,  // 最后输入时间（防抖）
}

impl NavigationState {
    /// 创建新导航状态（参考地牢物品栏默认8格）
    pub fn new(item_count: usize) -> Self {
        Self {
            current_focus: 0,
            item_count: item_count.max(1), // 确保至少1个项目
            wrap_around: true,
            grid_width: None,
            last_input_time: Instant::now(),
        }
    }

    /// 设置网格布局（用于物品栏等网格状UI）
    pub fn set_grid(&mut self, width: usize) {
        self.grid_width = Some(width);
    }

    /// 处理导航输入（带200ms防抖）
    pub fn navigate(&mut self, direction: NavDirection) -> bool {
        let is_first_input = self.last_input_time == Instant::now();
        let now = Instant::now();
        if !is_first_input && now.duration_since(self.last_input_time) < Duration::from_millis(200)
        {
            return false;
        }
        self.last_input_time = now;

        let moved = match direction {
            // 垂直导航（考虑网格布局）
            NavDirection::Up => self.move_vertical(-1),
            NavDirection::Down => self.move_vertical(1),

            // 水平导航
            NavDirection::Left => self.move_horizontal(-1),
            NavDirection::Right => self.move_horizontal(1),

            // 线性导航
            NavDirection::Next => self.move_linear(1),
            NavDirection::Prev => self.move_linear(-1),
        };

        if moved {
            self.current_focus = self.current_focus.min(self.item_count - 1);
        }
        moved
    }

    /// 垂直移动（考虑网格布局）
    fn move_vertical(&mut self, delta: isize) -> bool {
        if let Some(width) = self.grid_width {
            let row = self.current_focus / width;
            let col = self.current_focus % width;

            let new_row = if delta > 0 {
                row + 1
            } else {
                row.saturating_sub(1)
            };

            let new_index = new_row.saturating_mul(width).saturating_add(col);
            if new_index < self.item_count {
                self.current_focus = new_index;
                return true;
            }
        }
        false
    }

    /// 水平移动
    fn move_horizontal(&mut self, delta: isize) -> bool {
        if let Some(width) = self.grid_width {
            let new_index = if delta > 0 {
                self.current_focus + 1
            } else {
                self.current_focus.saturating_sub(1)
            };

            if new_index < self.item_count && (new_index / width == self.current_focus / width) {
                self.current_focus = new_index;
                return true;
            }
        }
        false
    }

    /// 线性移动（用于列表导航）
    fn move_linear(&mut self, delta: isize) -> bool {
        let new_index = if delta > 0 {
            if self.current_focus < self.item_count - 1 {
                self.current_focus + 1
            } else if self.wrap_around {
                0
            } else {
                self.current_focus
            }
        } else {
            if self.current_focus > 0 {
                self.current_focus - 1
            } else if self.wrap_around {
                self.item_count - 1
            } else {
                self.current_focus
            }
        };

        if new_index != self.current_focus {
            self.current_focus = new_index;
            true
        } else {
            false
        }
    }

    /// 获取当前聚焦项索引
    pub fn current(&self) -> usize {
        self.current_focus
    }

    /// 设置是否允许循环导航（如主菜单循环 vs 物品栏不循环）
    pub fn set_wrap_around(&mut self, wrap: bool) {
        self.wrap_around = wrap;
    }

    /// 直接跳转到指定索引（用于快捷键）
    pub fn jump_to(&mut self, index: usize) -> bool {
        if index < self.item_count {
            self.current_focus = index;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEvent, KeyModifiers};

    #[test]
    fn test_grid_navigation() {
        let mut nav = NavigationState::new(8);
        nav.set_grid(4);

        // 测试向右移动
        assert!(nav.navigate(NavDirection::Right));
        assert_eq!(nav.current(), 1);

        // 测试向下移动
        assert!(nav.navigate(NavDirection::Down));
        assert_eq!(nav.current(), 5);

        // 测试边界
        nav.jump_to(7);
        assert!(!nav.navigate(NavDirection::Right)); // 最右列不能右移
        assert!(nav.navigate(NavDirection::Down)); // 但可以下移（循环）
        assert_eq!(nav.current(), 3);
    }

    #[test]
    fn test_key_mapping() {
        let event = Event::Key(KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE));
        assert_eq!(NavDirection::from_event(&event), Some(NavDirection::Up));

        let tab_event = Event::Key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        assert_eq!(
            NavDirection::from_event(&tab_event),
            Some(NavDirection::Next)
        );
    }
}

#[test]
fn test_single_column_grid() {
    let mut nav = NavigationState::new(5);
    nav.set_grid(1); // 单列
    assert!(nav.navigate(NavDirection::Down));
    assert_eq!(nav.current(), 1);
}

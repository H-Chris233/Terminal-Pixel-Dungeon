//! Input handling abstractions for the ECS architecture.

use crate::ecs::*;
use crate::ecs::{NavigateDirection, PlayerAction};
use anyhow;
use crossterm::event::{
    self, Event as CEvent, KeyCode as CrosstermKeyCode, KeyEvent as CrosstermKeyEvent,
    KeyModifiers as CrosstermKeyModifiers,
};
use std::time::Duration;

/// Trait for input sources
pub trait InputSource {
    type Event;

    /// Poll for input events with a timeout
    fn poll(&mut self, timeout: Duration) -> anyhow::Result<Option<Self::Event>>;

    /// Check if input is available without blocking
    fn is_input_available(&self) -> anyhow::Result<bool>;
}

/// Console input source implementation
pub struct ConsoleInput {
    // Could include stdin handle or other input mechanisms
}

impl ConsoleInput {
    pub fn new() -> Self {
        Self {}
    }

    /// Process input events and convert them to PlayerActions for the ECS
    pub fn process_events(&mut self, resources: &mut Resources) -> anyhow::Result<()> {
        // Process crossterm events if available
        if event::poll(Duration::from_millis(50))? {
            if let Ok(CEvent::Key(key_event)) = event::read() {
                if let Some(action) =
                    key_event_to_player_action(key_event, &resources.game_state.game_state)
                {
                    resources.input_buffer.pending_actions.push(action);
                }
            }
        }

        Ok(())
    }
}

impl InputSource for ConsoleInput {
    type Event = InputEvent;

    fn poll(&mut self, timeout: Duration) -> anyhow::Result<Option<Self::Event>> {
        // In a real implementation, we would use crossterm or similar to poll for events
        // For now, we'll just return None to indicate no input
        if event::poll(timeout)? {
            if let Ok(event) = event::read() {
                return Ok(Some(InputEvent::from(event)));
            }
        }
        Ok(None)
    }

    fn is_input_available(&self) -> anyhow::Result<bool> {
        // In a real implementation, we would check if input is ready
        Ok(event::poll(Duration::from_millis(0))?)
    }
}

/// Terminal input events
#[derive(Debug, Clone)]
pub enum InputEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
}

/// Key events
#[derive(Debug, Clone)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

/// Mouse events
#[derive(Debug, Clone)]
pub enum MouseEvent {
    Press(MouseButton, u16, u16),
    Release(u16, u16),
    Hold(u16, u16),
}

/// Key codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCode {
    Char(char),
    Enter,
    Esc,
    Backspace,
    Delete,
    Tab,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    Insert,
    F(u8),
    Null,
}

/// Key modifiers
#[derive(Debug, Clone, Copy, Default)]
pub struct KeyModifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

/// Mouse buttons
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// Convert crossterm events to our internal events
impl From<crossterm::event::Event> for InputEvent {
    fn from(event: crossterm::event::Event) -> Self {
        match event {
            crossterm::event::Event::Key(key_event) => InputEvent::Key(KeyEvent::from(key_event)),
            crossterm::event::Event::Mouse(mouse_event) => {
                InputEvent::Mouse(MouseEvent::from(mouse_event))
            }
            crossterm::event::Event::Resize(width, height) => InputEvent::Resize(width, height),
            _ => InputEvent::Resize(0, 0), // fallback
        }
    }
}

impl From<crossterm::event::KeyEvent> for KeyEvent {
    fn from(key_event: crossterm::event::KeyEvent) -> Self {
        Self {
            code: KeyCode::from(key_event.code),
            modifiers: KeyModifiers::from(key_event.modifiers),
        }
    }
}

impl From<crossterm::event::KeyCode> for KeyCode {
    fn from(code: crossterm::event::KeyCode) -> Self {
        match code {
            crossterm::event::KeyCode::Char(c) => KeyCode::Char(c),
            crossterm::event::KeyCode::Enter => KeyCode::Enter,
            crossterm::event::KeyCode::Esc => KeyCode::Esc,
            crossterm::event::KeyCode::Backspace => KeyCode::Backspace,
            crossterm::event::KeyCode::Delete => KeyCode::Delete,
            crossterm::event::KeyCode::Tab => KeyCode::Tab,
            crossterm::event::KeyCode::Up => KeyCode::Up,
            crossterm::event::KeyCode::Down => KeyCode::Down,
            crossterm::event::KeyCode::Left => KeyCode::Left,
            crossterm::event::KeyCode::Right => KeyCode::Right,
            crossterm::event::KeyCode::Home => KeyCode::Home,
            crossterm::event::KeyCode::End => KeyCode::End,
            crossterm::event::KeyCode::PageUp => KeyCode::PageUp,
            crossterm::event::KeyCode::PageDown => KeyCode::PageDown,
            crossterm::event::KeyCode::Insert => KeyCode::Insert,
            crossterm::event::KeyCode::F(n) => KeyCode::F(n),
            _ => KeyCode::Null,
        }
    }
}

impl From<crossterm::event::KeyModifiers> for KeyModifiers {
    fn from(modifiers: crossterm::event::KeyModifiers) -> Self {
        Self {
            shift: modifiers.contains(crossterm::event::KeyModifiers::SHIFT),
            ctrl: modifiers.contains(crossterm::event::KeyModifiers::CONTROL),
            alt: modifiers.contains(crossterm::event::KeyModifiers::ALT),
        }
    }
}

impl From<crossterm::event::MouseEvent> for MouseEvent {
    fn from(event: crossterm::event::MouseEvent) -> Self {
        match event.kind {
            crossterm::event::MouseEventKind::Down(button) => {
                MouseEvent::Press(MouseButton::from(button), event.column, event.row)
            }
            crossterm::event::MouseEventKind::Up(_) => MouseEvent::Release(event.column, event.row),
            crossterm::event::MouseEventKind::Drag(_) => MouseEvent::Hold(event.column, event.row),
            _ => MouseEvent::Hold(event.column, event.row), // fallback
        }
    }
}

impl From<crossterm::event::MouseButton> for MouseButton {
    fn from(button: crossterm::event::MouseButton) -> Self {
        match button {
            crossterm::event::MouseButton::Left => MouseButton::Left,
            crossterm::event::MouseButton::Right => MouseButton::Right,
            crossterm::event::MouseButton::Middle => MouseButton::Middle,
        }
    }
}

/// Convert crossterm key events to player actions for the ECS
pub fn key_event_to_player_action(
    key: CrosstermKeyEvent,
    game_state: &crate::ecs::GameStatus,
) -> Option<PlayerAction> {
    // 根据游戏状态决定如何解释按键
    match game_state {
        crate::ecs::GameStatus::MainMenu { .. }
        | crate::ecs::GameStatus::Paused
        | crate::ecs::GameStatus::Options { .. }
        | crate::ecs::GameStatus::Inventory { .. }
        | crate::ecs::GameStatus::Help
        | crate::ecs::GameStatus::CharacterInfo => {
            // 在菜单状态下，按键被解释为菜单导航
            match_key_for_menu_context(key)
        }
        _ => {
            // 在游戏状态下，按键被解释为游戏控制
            match_key_for_game_context(key)
        }
    }
}

/// 处理菜单上下文中的按键
fn match_key_for_menu_context(key: CrosstermKeyEvent) -> Option<PlayerAction> {
    match key.code {
        // 菜单导航（支持方向键、vi-keys、WASD）
        CrosstermKeyCode::Up
        | CrosstermKeyCode::Char('k')
        | CrosstermKeyCode::Char('w') => Some(PlayerAction::MenuNavigate(NavigateDirection::Up)),
        CrosstermKeyCode::Down
        | CrosstermKeyCode::Char('j')
        | CrosstermKeyCode::Char('s') => Some(PlayerAction::MenuNavigate(NavigateDirection::Down)),
        CrosstermKeyCode::Left
        | CrosstermKeyCode::Char('h')
        | CrosstermKeyCode::Char('a') => Some(PlayerAction::MenuNavigate(NavigateDirection::Left)),
        CrosstermKeyCode::Right
        | CrosstermKeyCode::Char('l')
        | CrosstermKeyCode::Char('d') => Some(PlayerAction::MenuNavigate(NavigateDirection::Right)),
        CrosstermKeyCode::PageUp => Some(PlayerAction::MenuNavigate(NavigateDirection::PageUp)),
        CrosstermKeyCode::PageDown => Some(PlayerAction::MenuNavigate(NavigateDirection::PageDown)),

        // 菜单确认和返回
        CrosstermKeyCode::Enter => Some(PlayerAction::MenuSelect),
        CrosstermKeyCode::Esc | CrosstermKeyCode::Backspace => Some(PlayerAction::CloseMenu),

        // 在菜单中也支持一些快捷键
        CrosstermKeyCode::Char('i') => Some(PlayerAction::OpenInventory),
        CrosstermKeyCode::Char('o') => Some(PlayerAction::OpenOptions),
        CrosstermKeyCode::Char('?') => Some(PlayerAction::OpenHelp),
        CrosstermKeyCode::Char('c') => Some(PlayerAction::OpenCharacterInfo),
        CrosstermKeyCode::Char('q') => Some(PlayerAction::Quit),

        _ => None,
    }
}

/// 处理游戏上下文中的按键
fn match_key_for_game_context(key: CrosstermKeyEvent) -> Option<PlayerAction> {
    match (key.code, key.modifiers) {
        // Movement keys（支持方向键、vi-keys、完整 WASD）
        (CrosstermKeyCode::Char('k'), _)
        | (CrosstermKeyCode::Up, _)
        | (CrosstermKeyCode::Char('w'), _) => Some(PlayerAction::Move(Direction::North)),
        (CrosstermKeyCode::Char('j'), _)
        | (CrosstermKeyCode::Down, _)
        | (CrosstermKeyCode::Char('s'), _) => Some(PlayerAction::Move(Direction::South)),
        (CrosstermKeyCode::Char('h'), _)
        | (CrosstermKeyCode::Left, _)
        | (CrosstermKeyCode::Char('a'), _) => Some(PlayerAction::Move(Direction::West)),
        (CrosstermKeyCode::Char('l'), _)
        | (CrosstermKeyCode::Right, _)
        | (CrosstermKeyCode::Char('d'), _) => Some(PlayerAction::Move(Direction::East)),
        (CrosstermKeyCode::Char('y'), _) => Some(PlayerAction::Move(Direction::NorthWest)),
        (CrosstermKeyCode::Char('u'), _) => Some(PlayerAction::Move(Direction::NorthEast)),
        (CrosstermKeyCode::Char('b'), _) => Some(PlayerAction::Move(Direction::SouthWest)),
        (CrosstermKeyCode::Char('n'), _) => Some(PlayerAction::Move(Direction::SouthEast)),

        // Wait/skip turn
        (CrosstermKeyCode::Char('.'), _) => Some(PlayerAction::Wait),

        // Stairs
        (CrosstermKeyCode::Char('>'), _) => Some(PlayerAction::Descend),
        (CrosstermKeyCode::Char('<'), _) => Some(PlayerAction::Ascend),

        // Attack via direction（支持 vi-keys 和 WASD 的 Shift 组合）
        (CrosstermKeyCode::Char('K'), _)
        | (CrosstermKeyCode::Char('W'), _) => Some(PlayerAction::Attack(Position { x: 0, y: -1, z: 0 })),
        (CrosstermKeyCode::Char('J'), _)
        | (CrosstermKeyCode::Char('S'), _) => Some(PlayerAction::Attack(Position { x: 0, y: 1, z: 0 })),
        (CrosstermKeyCode::Char('H'), _)
        | (CrosstermKeyCode::Char('A'), _) => Some(PlayerAction::Attack(Position { x: -1, y: 0, z: 0 })),
        (CrosstermKeyCode::Char('L'), _)
        | (CrosstermKeyCode::Char('D'), _) => Some(PlayerAction::Attack(Position { x: 1, y: 0, z: 0 })),
        (CrosstermKeyCode::Char('Y'), _) => Some(PlayerAction::Attack(Position { x: -1, y: -1, z: 0 })),
        (CrosstermKeyCode::Char('U'), _) => Some(PlayerAction::Attack(Position { x: 1, y: -1, z: 0 })),
        (CrosstermKeyCode::Char('B'), _) => Some(PlayerAction::Attack(Position { x: -1, y: 1, z: 0 })),
        (CrosstermKeyCode::Char('N'), _) => Some(PlayerAction::Attack(Position { x: 1, y: 1, z: 0 })),

        // Game control
        (CrosstermKeyCode::Char('q'), _) => Some(PlayerAction::Quit),

        // Number keys for items/spells (for later implementation)
        (CrosstermKeyCode::Char('1'), _) => Some(PlayerAction::UseItem(0)),
        (CrosstermKeyCode::Char('2'), _) => Some(PlayerAction::UseItem(1)),
        (CrosstermKeyCode::Char('3'), _) => Some(PlayerAction::UseItem(2)),
        (CrosstermKeyCode::Char('4'), _) => Some(PlayerAction::UseItem(3)),
        (CrosstermKeyCode::Char('5'), _) => Some(PlayerAction::UseItem(4)),
        (CrosstermKeyCode::Char('6'), _) => Some(PlayerAction::UseItem(5)),
        (CrosstermKeyCode::Char('7'), _) => Some(PlayerAction::UseItem(6)),
        (CrosstermKeyCode::Char('8'), _) => Some(PlayerAction::UseItem(7)),
        (CrosstermKeyCode::Char('9'), _) => Some(PlayerAction::UseItem(8)),

        // Drop item - 现在使用 Delete 键而不是 'd' 键
        (CrosstermKeyCode::Delete, _) => Some(PlayerAction::DropItem(0)), // Default to first item

        // 游戏中的快捷键
        (CrosstermKeyCode::Char('i'), _) => Some(PlayerAction::OpenInventory),
        (CrosstermKeyCode::Char('o'), _) => Some(PlayerAction::OpenOptions),
        (CrosstermKeyCode::Char('?'), _) => Some(PlayerAction::OpenHelp),
        (CrosstermKeyCode::Char('c'), _) => Some(PlayerAction::OpenCharacterInfo),
        (CrosstermKeyCode::Esc, _) => Some(PlayerAction::CloseMenu), // 暂停游戏

        _ => None,
    }
}

/// Process input and update ECS world
pub fn process_input(ecs_world: &mut ECSWorld) -> anyhow::Result<bool> {
    // Process crossterm events and add to input buffer
    if event::poll(Duration::from_millis(50))? {
        if let Ok(CEvent::Key(key_event)) = event::read() {
            let current_game_state = &ecs_world.resources.game_state.game_state;
            if let Some(action) = key_event_to_player_action(key_event, current_game_state) {
                ecs_world
                    .resources
                    .input_buffer
                    .pending_actions
                    .push(action);
            }
        }
    }

    // Return true if we received a quit command
    Ok(ecs_world
        .resources
        .input_buffer
        .pending_actions
        .iter()
        .any(|action| matches!(action, PlayerAction::Quit)))
}

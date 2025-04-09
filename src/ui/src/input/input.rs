use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::{Duration, Instant};

/// 输入系统配置
#[derive(Debug, Clone)]
pub struct InputConfig {
    /// 按键重复初始延迟（毫秒）
    pub repeat_delay: u64,
    /// 按键重复间隔（毫秒）
    pub repeat_interval: u64,
    /// 快捷使用触发时间（毫秒）
    pub quick_use_threshold: u64,
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            repeat_delay: 400,        // 与像素地牢PC版一致
            repeat_interval: 30,      // 流畅的菜单导航
            quick_use_threshold: 300, // 快速使用物品阈值
        }
    }
}

/// 主输入处理系统
#[derive(Debug)]
pub struct InputSystem {
    // 状态跟踪
    last_key: Option<KeyEvent>,
    last_key_time: Instant,
    current_modifiers: KeyModifiers,

    // 配置
    config: InputConfig,

    // 特殊状态
    quick_use_state: Option<Instant>,
    last_processed: Instant,
}

impl Default for InputSystem {
    fn default() -> Self {
        Self {
            last_key: None,
            last_key_time: Instant::now(),
            current_modifiers: KeyModifiers::NONE,
            config: InputConfig::default(),
            quick_use_state: None,
            last_processed: Instant::now(),
        }
    }
}

impl InputSystem {
    /// 更新配置
    pub fn configure(&mut self, config: InputConfig) {
        self.config = config;
    }

    /// 检查按键匹配（无修饰键）
    pub fn match_key(&self, event: &Event, key: KeyCode) -> bool {
        matches!(
            event,
            Event::Key(KeyEvent {
                code,
                modifiers: KeyModifiers::NONE,
                ..
            }) if *code == key
        )
    }

    /// 检查组合键（带修饰键）
    pub fn match_combo(&self, event: &Event, key: KeyCode, modifier: KeyModifiers) -> bool {
        matches!(
            event,
            Event::Key(KeyEvent {
                code,
                modifiers,
                ..
            }) if *code == key && *modifiers == modifier
        )
    }

    /// 处理数字键快捷栏（1-9对应0-8，0对应9）
    pub fn get_quick_slot(&self, event: &Event) -> Option<usize> {
        if let Event::Key(KeyEvent {
            code: KeyCode::Char(c),
            modifiers: KeyModifiers::NONE,
            ..
        }) = event
        {
            match c {
                '1'..='9' => Some(*c as usize - '1' as usize),
                '0' => Some(9),
                _ => None,
            }
        } else {
            None
        }
    }

    /// 处理按键重复（用于移动和菜单导航）
    pub fn should_repeat(&mut self, event: &Event) -> bool {
        let now = Instant::now();
        let mut should_repeat = false;

        if let Event::Key(key_event) = event {
            // 检查重复条件
            if let Some(last_key) = self.last_key {
                if *key_event == last_key {
                    let elapsed = now - self.last_key_time;

                    // 超过初始延迟后开始重复
                    if elapsed > Duration::from_millis(self.config.repeat_delay) {
                        should_repeat = (now - self.last_processed)
                            >= Duration::from_millis(self.config.repeat_interval);
                    }
                }
            }

            // 更新状态
            self.last_key = Some(*key_event);
            self.last_key_time = now;
            self.current_modifiers = key_event.modifiers;

            if should_repeat {
                self.last_processed = now;
            }
        }

        should_repeat
    }

    /// 开始快捷使用模式（如连续喝药水）
    pub fn begin_quick_use(&mut self) {
        self.quick_use_state = Some(Instant::now());
    }

    /// 结束快捷使用模式
    pub fn end_quick_use(&mut self) {
        self.quick_use_state = None;
    }

    /// 检查是否触发快捷使用
    pub fn should_quick_use(&self) -> bool {
        self.quick_use_state.map_or(false, |start| {
            Instant::now() - start > Duration::from_millis(self.config.quick_use_threshold)
        })
    }

    /// 轮询输入事件（带超时）
    pub fn poll_event(&self, timeout: Option<Duration>) -> std::io::Result<Option<Event>> {
        match timeout {
            Some(dur) if crossterm::event::poll(dur)? => crossterm::event::read().map(Some),
            Some(_) => Ok(None),
            None => crossterm::event::read().map(Some),
        }
    }

    /// 获取当前修饰键状态
    pub fn modifiers(&self) -> KeyModifiers {
        self.current_modifiers
    }

    /// 重置输入状态（切换场景时调用）
    pub fn reset(&mut self) {
        self.last_key = None;
        self.quick_use_state = None;
        self.last_processed = Instant::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyEventKind;

    fn test_key_event(code: KeyCode) -> Event {
        Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
    }

    #[test]
    fn test_basic_input() {
        let mut input = InputSystem::default();
        let event = test_key_event(KeyCode::Char('g'));

        assert!(input.match_key(&event, KeyCode::Char('g')));
        assert!(!input.should_repeat(&event)); // 首次按下不重复
    }

    #[test]
    fn test_quick_slots() {
        let input = InputSystem::default();

        assert_eq!(
            input.get_quick_slot(&test_key_event(KeyCode::Char('1'))),
            Some(0)
        );
        assert_eq!(
            input.get_quick_slot(&test_key_event(KeyCode::Char('0'))),
            Some(9)
        );
        assert_eq!(input.get_quick_slot(&test_key_event(KeyCode::Up)), None);
    }

    #[test]
    fn test_quick_use() {
        let mut input = InputSystem::default();
        input.begin_quick_use();

        // 立即检查不应触发
        assert!(!input.should_quick_use());

        // 模拟时间流逝
        input.config.quick_use_threshold = 0;
        assert!(input.should_quick_use());
    }

    #[test]
    fn test_key_repeat() {
        let mut input = InputSystem {
            config: InputConfig {
                repeat_delay: 0, // 立即开始重复
                repeat_interval: 10,
                ..Default::default()
            },
            ..Default::default()
        };

        let event = test_key_event(KeyCode::Down);
        assert!(!input.should_repeat(&event)); // 首次按下

        // 模拟快速重复
        assert!(input.should_repeat(&event));
    }
}

//! 增强输入系统
//!
//! 提供更丰富的输入处理功能：
//! - 按键组合和序列
//! - 输入历史和重复
//! - 自定义按键映射
//! - 上下文敏感的输入处理

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

/// 扩展的输入事件
#[derive(Debug, Clone)]
pub enum EnhancedInputEvent {
    /// 单个按键
    KeyPress(KeyEvent),
    /// 按键组合 (Ctrl+C, Alt+F4 等)
    KeyCombo(Vec<KeyEvent>),
    /// 按键序列 (vim风格的 :wq 等)
    KeySequence(Vec<KeyEvent>),
    /// 鼠标事件
    Mouse(MouseEvent),
    /// 长按事件
    KeyHold(KeyEvent, Duration),
    /// 双击事件
    DoubleClick(KeyEvent),
    /// 文本输入 (处理过的字符串)
    TextInput(String),
}

/// 输入模式
#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    /// 游戏模式 - 标准游戏控制
    Game,
    /// 菜单模式 - 菜单导航
    Menu,
    /// 文本输入模式
    TextInput,
    /// 命令模式 (类似vim的命令模式)
    Command,
    /// 快捷键模式 (等待快捷键输入)
    Shortcut,
    /// 确认模式 (等待确认输入)
    Confirmation,
}

/// 按键映射配置
#[derive(Debug, Clone)]
pub struct KeyMapping {
    /// 按键到动作的映射
    pub mappings: HashMap<String, String>,
    /// 按键序列映射 (multi-key bindings)
    pub sequence_mappings: HashMap<Vec<String>, String>,
    /// 上下文特定的映射
    pub context_mappings: HashMap<InputMode, HashMap<String, String>>,
}

impl KeyMapping {
    pub fn new() -> Self {
        let mut mapping = Self {
            mappings: HashMap::new(),
            sequence_mappings: HashMap::new(),
            context_mappings: HashMap::new(),
        };
        mapping.load_default_mappings();
        mapping
    }

    /// 加载默认按键映射
    fn load_default_mappings(&mut self) {
        // 游戏模式映射
        let mut game_mappings = HashMap::new();
        
        // 移动
        game_mappings.insert("h".to_string(), "move_west".to_string());
        game_mappings.insert("j".to_string(), "move_south".to_string());
        game_mappings.insert("k".to_string(), "move_north".to_string());
        game_mappings.insert("l".to_string(), "move_east".to_string());
        game_mappings.insert("y".to_string(), "move_northwest".to_string());
        game_mappings.insert("u".to_string(), "move_northeast".to_string());
        game_mappings.insert("b".to_string(), "move_southwest".to_string());
        game_mappings.insert("n".to_string(), "move_southeast".to_string());
        
        // 动作
        game_mappings.insert(".".to_string(), "wait".to_string());
        game_mappings.insert(">".to_string(), "descend".to_string());
        game_mappings.insert("<".to_string(), "ascend".to_string());
        game_mappings.insert("g".to_string(), "pickup".to_string());
        game_mappings.insert("d".to_string(), "drop".to_string());
        
        // 界面
        game_mappings.insert("i".to_string(), "inventory".to_string());
        game_mappings.insert("c".to_string(), "character".to_string());
        game_mappings.insert("?".to_string(), "help".to_string());
        game_mappings.insert("m".to_string(), "messages".to_string());
        
        // 快捷键
        for i in 1..=9 {
            game_mappings.insert(i.to_string(), format!("quickslot_{}", i));
        }
        
        self.context_mappings.insert(InputMode::Game, game_mappings);
        
        // 菜单模式映射
        let mut menu_mappings = HashMap::new();
        menu_mappings.insert("j".to_string(), "menu_down".to_string());
        menu_mappings.insert("k".to_string(), "menu_up".to_string());
        menu_mappings.insert("h".to_string(), "menu_left".to_string());
        menu_mappings.insert("l".to_string(), "menu_right".to_string());
        menu_mappings.insert("Enter".to_string(), "menu_select".to_string());
        menu_mappings.insert("Escape".to_string(), "menu_back".to_string());
        
        self.context_mappings.insert(InputMode::Menu, menu_mappings);
        
        // 按键序列映射 (vim风格)
        self.sequence_mappings.insert(
            vec![":".to_string(), "q".to_string()], 
            "quit".to_string()
        );
        self.sequence_mappings.insert(
            vec![":".to_string(), "w".to_string()], 
            "save".to_string()
        );
        self.sequence_mappings.insert(
            vec![":".to_string(), "w".to_string(), "q".to_string()], 
            "save_and_quit".to_string()
        );
    }

    /// 根据上下文获取动作
    pub fn get_action(&self, key_str: &str, mode: InputMode) -> Option<String> {
        // 首先检查上下文特定的映射
        if let Some(context_map) = self.context_mappings.get(&mode) {
            if let Some(action) = context_map.get(key_str) {
                return Some(action.clone());
            }
        }
        
        // 然后检查全局映射
        self.mappings.get(key_str).cloned()
    }

    /// 更新按键映射
    pub fn set_mapping(&mut self, key: String, action: String, mode: Option<InputMode>) {
        if let Some(mode) = mode {
            self.context_mappings
                .entry(mode)
                .or_insert_with(HashMap::new)
                .insert(key, action);
        } else {
            self.mappings.insert(key, action);
        }
    }
}

/// 输入历史管理
#[derive(Debug)]
pub struct InputHistory {
    /// 输入事件历史
    events: VecDeque<(EnhancedInputEvent, Instant)>,
    /// 文本输入历史
    text_history: VecDeque<String>,
    /// 最大历史记录数
    max_events: usize,
    max_text: usize,
}

impl InputHistory {
    pub fn new() -> Self {
        Self {
            events: VecDeque::with_capacity(100),
            text_history: VecDeque::with_capacity(50),
            max_events: 100,
            max_text: 50,
        }
    }

    /// 添加输入事件
    pub fn add_event(&mut self, event: EnhancedInputEvent) {
        self.events.push_back((event, Instant::now()));
        while self.events.len() > self.max_events {
            self.events.pop_front();
        }
    }

    /// 添加文本输入
    pub fn add_text(&mut self, text: String) {
        if !text.is_empty() && self.text_history.back() != Some(&text) {
            self.text_history.push_back(text);
            while self.text_history.len() > self.max_text {
                self.text_history.pop_front();
            }
        }
    }

    /// 获取最近的事件
    pub fn get_recent_events(&self, count: usize) -> Vec<&EnhancedInputEvent> {
        self.events
            .iter()
            .rev()
            .take(count)
            .map(|(event, _)| event)
            .collect()
    }

    /// 获取文本历史
    pub fn get_text_history(&self) -> &VecDeque<String> {
        &self.text_history
    }

    /// 检查是否有重复的按键序列
    pub fn detect_repeat_pattern(&self, window_size: usize) -> Option<Vec<EnhancedInputEvent>> {
        if self.events.len() < window_size * 2 {
            return None;
        }

        let recent_events: Vec<_> = self.events
            .iter()
            .rev()
            .take(window_size * 2)
            .map(|(event, _)| event.clone())
            .collect();

        let first_half = &recent_events[window_size..];
        let second_half = &recent_events[..window_size];

        if first_half == second_half {
            Some(first_half.to_vec())
        } else {
            None
        }
    }
}

/// 增强输入处理器
pub struct EnhancedInputProcessor {
    /// 当前输入模式
    mode: InputMode,
    /// 按键映射
    key_mapping: KeyMapping,
    /// 输入历史
    history: InputHistory,
    /// 当前按键序列缓冲
    sequence_buffer: Vec<String>,
    /// 等待组合键的状态
    combo_state: HashMap<KeyCode, Instant>,
    /// 长按检测
    hold_tracker: HashMap<KeyCode, Instant>,
    /// 双击检测
    double_click_tracker: HashMap<KeyCode, (Instant, u8)>,
    /// 输入超时设置
    sequence_timeout: Duration,
    combo_timeout: Duration,
    double_click_timeout: Duration,
    hold_threshold: Duration,
}

impl EnhancedInputProcessor {
    pub fn new() -> Self {
        Self {
            mode: InputMode::Game,
            key_mapping: KeyMapping::new(),
            history: InputHistory::new(),
            sequence_buffer: Vec::new(),
            combo_state: HashMap::new(),
            hold_tracker: HashMap::new(),
            double_click_tracker: HashMap::new(),
            sequence_timeout: Duration::from_secs(2),
            combo_timeout: Duration::from_millis(500),
            double_click_timeout: Duration::from_millis(300),
            hold_threshold: Duration::from_millis(500),
        }
    }

    /// 设置输入模式
    pub fn set_mode(&mut self, mode: InputMode) {
        if self.mode != mode {
            self.clear_buffers();
            self.mode = mode;
        }
    }

    /// 获取当前模式
    pub fn get_mode(&self) -> &InputMode {
        &self.mode
    }

    /// 处理按键事件
    pub fn process_key_event(&mut self, key_event: KeyEvent) -> Vec<EnhancedInputEvent> {
        let mut results = Vec::new();
        let now = Instant::now();

        // 更新长按状态
        if self.update_hold_state(key_event.code, now) {
            results.push(EnhancedInputEvent::KeyHold(key_event, now - self.hold_tracker[&key_event.code]));
        }

        // 检查双击
        if self.check_double_click(key_event.code, now) {
            results.push(EnhancedInputEvent::DoubleClick(key_event));
        }

        // 处理组合键
        if key_event.modifiers != KeyModifiers::NONE {
            self.combo_state.insert(key_event.code, now);
            results.push(EnhancedInputEvent::KeyPress(key_event));
        } else {
            // 处理按键序列
            let key_str = keyevent_to_string(&key_event);
            self.sequence_buffer.push(key_str.clone());

            // 检查是否有匹配的序列
            if let Some(_action) = self.check_sequence_match() {
                results.push(EnhancedInputEvent::KeySequence(vec![key_event]));
                self.sequence_buffer.clear();
            } else {
                results.push(EnhancedInputEvent::KeyPress(key_event));
            }
        }

        // 清理过期状态
        self.cleanup_expired_state(now);

        // 记录到历史
        for event in &results {
            self.history.add_event(event.clone());
        }

        results
    }

    /// 处理文本输入
    pub fn process_text_input(&mut self, text: String) -> EnhancedInputEvent {
        self.history.add_text(text.clone());
        EnhancedInputEvent::TextInput(text)
    }

    /// 获取动作映射
    pub fn get_action_for_key(&self, key_str: &str) -> Option<String> {
        self.key_mapping.get_action(key_str, self.mode.clone())
    }

    /// 更新按键映射
    pub fn update_key_mapping(&mut self, key: String, action: String) {
        self.key_mapping.set_mapping(key, action, Some(self.mode.clone()));
    }

    /// 获取输入历史
    pub fn get_history(&self) -> &InputHistory {
        &self.history
    }

    /// 清空所有缓冲区
    fn clear_buffers(&mut self) {
        self.sequence_buffer.clear();
        self.combo_state.clear();
        self.hold_tracker.clear();
        self.double_click_tracker.clear();
    }

    /// 更新长按状态
    fn update_hold_state(&mut self, key: KeyCode, now: Instant) -> bool {
        if let Some(start_time) = self.hold_tracker.get(&key) {
            now.duration_since(*start_time) >= self.hold_threshold
        } else {
            self.hold_tracker.insert(key, now);
            false
        }
    }

    /// 检查双击
    fn check_double_click(&mut self, key: KeyCode, now: Instant) -> bool {
        if let Some((last_time, count)) = self.double_click_tracker.get_mut(&key) {
            if now.duration_since(*last_time) <= self.double_click_timeout {
                *count += 1;
                *last_time = now;
                return *count >= 2;
            } else {
                *count = 1;
                *last_time = now;
            }
        } else {
            self.double_click_tracker.insert(key, (now, 1));
        }
        false
    }

    /// 检查按键序列匹配
    fn check_sequence_match(&self) -> Option<String> {
        for (sequence, action) in &self.key_mapping.sequence_mappings {
            if self.sequence_buffer == *sequence {
                return Some(action.clone());
            }
        }
        None
    }

    /// 清理过期状态
    fn cleanup_expired_state(&mut self, now: Instant) {
        // 清理过期的序列缓冲
        if let Some(_oldest) = self.sequence_buffer.first() {
            // 简化：如果缓冲区太长就清空
            if self.sequence_buffer.len() > 10 {
                self.sequence_buffer.clear();
            }
        }

        // 清理过期的组合键状态
        self.combo_state.retain(|_, time| now.duration_since(*time) <= self.combo_timeout);

        // 清理过期的双击状态
        self.double_click_tracker.retain(|_, (time, _)| {
            now.duration_since(*time) <= self.double_click_timeout
        });
    }
}

impl Default for EnhancedInputProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// 将KeyEvent转换为字符串
fn keyevent_to_string(key_event: &KeyEvent) -> String {
    let mut result = String::new();

    if key_event.modifiers.contains(KeyModifiers::CONTROL) {
        result.push_str("Ctrl+");
    }
    if key_event.modifiers.contains(KeyModifiers::ALT) {
        result.push_str("Alt+");
    }
    if key_event.modifiers.contains(KeyModifiers::SHIFT) {
        result.push_str("Shift+");
    }

    match key_event.code {
        KeyCode::Char(c) => result.push(c),
        KeyCode::Enter => result.push_str("Enter"),
        KeyCode::Esc => result.push_str("Escape"),
        KeyCode::Backspace => result.push_str("Backspace"),
        KeyCode::Delete => result.push_str("Delete"),
        KeyCode::Tab => result.push_str("Tab"),
        KeyCode::Up => result.push_str("Up"),
        KeyCode::Down => result.push_str("Down"),
        KeyCode::Left => result.push_str("Left"),
        KeyCode::Right => result.push_str("Right"),
        KeyCode::Home => result.push_str("Home"),
        KeyCode::End => result.push_str("End"),
        KeyCode::PageUp => result.push_str("PageUp"),
        KeyCode::PageDown => result.push_str("PageDown"),
        KeyCode::Insert => result.push_str("Insert"),
        KeyCode::F(n) => result.push_str(&format!("F{}", n)),
        _ => result.push_str("Unknown"),
    }

    result
}

/// 输入上下文管理器
pub struct InputContextManager {
    /// 上下文堆栈
    context_stack: Vec<InputMode>,
    /// 处理器
    processor: EnhancedInputProcessor,
}

impl InputContextManager {
    pub fn new() -> Self {
        Self {
            context_stack: vec![InputMode::Game],
            processor: EnhancedInputProcessor::new(),
        }
    }

    /// 推入新的输入上下文
    pub fn push_context(&mut self, mode: InputMode) {
        self.context_stack.push(mode.clone());
        self.processor.set_mode(mode);
    }

    /// 弹出当前输入上下文
    pub fn pop_context(&mut self) {
        if self.context_stack.len() > 1 {
            self.context_stack.pop();
            if let Some(previous_mode) = self.context_stack.last() {
                self.processor.set_mode(previous_mode.clone());
            }
        }
    }

    /// 获取当前上下文
    pub fn current_context(&self) -> &InputMode {
        self.context_stack.last().unwrap_or(&InputMode::Game)
    }

    /// 处理输入事件
    pub fn process_input(&mut self, key_event: KeyEvent) -> Vec<EnhancedInputEvent> {
        self.processor.process_key_event(key_event)
    }

    /// 获取处理器的引用
    pub fn processor(&self) -> &EnhancedInputProcessor {
        &self.processor
    }

    /// 获取处理器的可变引用
    pub fn processor_mut(&mut self) -> &mut EnhancedInputProcessor {
        &mut self.processor
    }
}

impl Default for InputContextManager {
    fn default() -> Self {
        Self::new()
    }
}
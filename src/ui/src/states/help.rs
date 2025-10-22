//! 帮助系统
//!
//! 提供游戏说明和按键帮助：
//! - 控制说明
//! - 游戏机制解释  
//! - 快捷键列表
//! - 分类浏览

use crate::input::{InputMode, KeyMapping};
use crate::render::animation::{Animation, AnimationManager, AnimationType, EaseType};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Tabs, Wrap},
    Frame,
};
use std::time::Duration;

/// 帮助主题分类
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HelpTopic {
    Controls,      // 控制说明
    Combat,        // 战斗机制
    Items,         // 物品系统
    Dungeon,       // 地牢探索
    Character,     // 角色系统
    Tips,          // 游戏技巧
    About,         // 关于游戏
}

impl HelpTopic {
    /// 获取所有主题
    pub fn all_topics() -> Vec<HelpTopic> {
        vec![
            HelpTopic::Controls,
            HelpTopic::Combat,
            HelpTopic::Items,
            HelpTopic::Dungeon,
            HelpTopic::Character,
            HelpTopic::Tips,
            HelpTopic::About,
        ]
    }

    /// 获取主题标题
    pub fn title(&self) -> &'static str {
        match self {
            HelpTopic::Controls => "Controls",
            HelpTopic::Combat => "Combat",
            HelpTopic::Items => "Items",
            HelpTopic::Dungeon => "Dungeon",
            HelpTopic::Character => "Character",
            HelpTopic::Tips => "Tips & Tricks",
            HelpTopic::About => "About",
        }
    }

    /// 获取主题图标
    pub fn icon(&self) -> char {
        match self {
            HelpTopic::Controls => '⌨',
            HelpTopic::Combat => '⚔',
            HelpTopic::Items => '🎒',
            HelpTopic::Dungeon => '🏰',
            HelpTopic::Character => '👤',
            HelpTopic::Tips => '💡',
            HelpTopic::About => 'ℹ',
        }
    }
}

/// 帮助条目
#[derive(Debug, Clone)]
pub struct HelpEntry {
    pub title: String,
    pub description: String,
    pub details: Vec<String>,
    pub key_bindings: Vec<(String, String)>, // (按键, 说明)
    pub examples: Vec<String>,
    pub related_topics: Vec<HelpTopic>,
}

impl HelpEntry {
    pub fn new(title: String, description: String) -> Self {
        Self {
            title,
            description,
            details: Vec::new(),
            key_bindings: Vec::new(),
            examples: Vec::new(),
            related_topics: Vec::new(),
        }
    }

    pub fn with_details(mut self, details: Vec<String>) -> Self {
        self.details = details;
        self
    }

    pub fn with_key_bindings(mut self, key_bindings: Vec<(String, String)>) -> Self {
        self.key_bindings = key_bindings;
        self
    }

    pub fn with_examples(mut self, examples: Vec<String>) -> Self {
        self.examples = examples;
        self
    }

    pub fn with_related_topics(mut self, related_topics: Vec<HelpTopic>) -> Self {
        self.related_topics = related_topics;
        self
    }
}

/// 帮助内容数据库
pub struct HelpDatabase {
    entries: std::collections::HashMap<HelpTopic, Vec<HelpEntry>>,
}

impl HelpDatabase {
    pub fn new() -> Self {
        let mut database = Self {
            entries: std::collections::HashMap::new(),
        };
        database.initialize_content();
        database
    }

    /// 初始化帮助内容
    fn initialize_content(&mut self) {
        // 控制说明
        let controls_entries = vec![
            HelpEntry::new(
                "Movement".to_string(),
                "Move your character around the dungeon".to_string(),
            )
            .with_key_bindings(vec![
                ("hjkl".to_string(), "Move left/down/up/right (vi-keys)".to_string()),
                ("yubn".to_string(), "Move diagonally".to_string()),
                ("Arrow Keys".to_string(), "Alternative movement".to_string()),
                ("WASD".to_string(), "Complete WASD movement support".to_string()),
            ])
            .with_details(vec![
                "Use hjkl keys for precise movement (vim style)".to_string(),
                "Full WASD support: W/A/S/D for up/left/down/right".to_string(),
                "Diagonal movement uses yubn keys".to_string(),
                "Moving into enemies will attack them".to_string(),
                "Moving into walls will do nothing".to_string(),
            ]),

            HelpEntry::new(
                "Actions".to_string(),
                "Interact with the dungeon and items".to_string(),
            )
            .with_key_bindings(vec![
                (".".to_string(), "Wait/Skip turn".to_string()),
                ("g".to_string(), "Pick up items".to_string()),
                ("Del".to_string(), "Drop items".to_string()),
                (">".to_string(), "Descend stairs".to_string()),
                ("<".to_string(), "Ascend stairs".to_string()),
            ]),

            HelpEntry::new(
                "Interface".to_string(),
                "Open menus and interface screens".to_string(),
            )
            .with_key_bindings(vec![
                ("i".to_string(), "Open inventory".to_string()),
                ("c".to_string(), "Character information".to_string()),
                ("?".to_string(), "Help (this screen)".to_string()),
                ("m".to_string(), "Message history".to_string()),
                ("ESC".to_string(), "Pause game / Back".to_string()),
                ("q".to_string(), "Quit game".to_string()),
            ]),

            HelpEntry::new(
                "Quick Actions".to_string(),
                "Use numbered quickslots for items".to_string(),
            )
            .with_key_bindings(vec![
                ("1-9".to_string(), "Use item in quickslot".to_string()),
                ("SHIFT+HJKL/WASD".to_string(), "Attack in direction".to_string()),
            ]),
        ];
        self.entries.insert(HelpTopic::Controls, controls_entries);

        // 战斗机制
        let combat_entries = vec![
            HelpEntry::new(
                "Combat Basics".to_string(),
                "How fighting works in Terminal Pixel Dungeon".to_string(),
            )
            .with_details(vec![
                "Move into enemies to attack them".to_string(),
                "Combat is turn-based - you act, then enemies act".to_string(),
                "Accuracy affects hit chance, Evasion helps avoid attacks".to_string(),
                "Defense reduces incoming damage".to_string(),
            ])
            .with_examples(vec![
                "Base hit chance is 80%, modified by accuracy vs evasion".to_string(),
                "Minimum hit chance is 5%, maximum is 95%".to_string(),
                "Critical hits deal 1.5x damage".to_string(),
            ]),

            HelpEntry::new(
                "Stealth Attacks".to_string(),
                "Attack enemies from outside their vision".to_string(),
            )
            .with_details(vec![
                "Attacking from outside enemy vision deals 2x damage".to_string(),
                "Enemies have limited field of view".to_string(),
                "Use walls and corners to stay hidden".to_string(),
                "Some enemies have better vision than others".to_string(),
            ]),

            HelpEntry::new(
                "Status Effects".to_string(),
                "Temporary conditions affecting combat".to_string(),
            )
            .with_details(vec![
                "Burning: Deals damage over time".to_string(),
                "Poisoned: Reduces health gradually".to_string(),
                "Bleeding: Continuous health loss".to_string(),
                "Paralyzed: Cannot move or act".to_string(),
                "Invisible: Enemies cannot see you".to_string(),
                "Slowed: Move and act less frequently".to_string(),
                "Haste: Move and act more frequently".to_string(),
            ]),
        ];
        self.entries.insert(HelpTopic::Combat, combat_entries);

        // 物品系统
        let items_entries = vec![
            HelpEntry::new(
                "Item Types".to_string(),
                "Different categories of items you can find".to_string(),
            )
            .with_details(vec![
                "Weapons: Swords, maces, bows, wands for combat".to_string(),
                "Armor: Protection from enemy attacks".to_string(),
                "Potions: Consumable items with various effects".to_string(),
                "Scrolls: Magic scrolls with powerful one-time effects".to_string(),
                "Rings: Provide passive bonuses when worn".to_string(),
                "Food: Restores hunger and sometimes provides benefits".to_string(),
            ]),

            HelpEntry::new(
                "Equipment".to_string(),
                "How to equip and use weapons and armor".to_string(),
            )
            .with_details(vec![
                "Equip weapons and armor from the inventory".to_string(),
                "Only one weapon and armor can be equipped at a time".to_string(),
                "Rings can be equipped (usually 2 ring slots)".to_string(),
                "Equipment affects your combat stats".to_string(),
            ]),

            HelpEntry::new(
                "Inventory Management".to_string(),
                "Organizing and using your items".to_string(),
            )
            .with_key_bindings(vec![
                ("i".to_string(), "Open inventory screen".to_string()),
                ("1-9".to_string(), "Quick-use items in slots".to_string()),
                ("d".to_string(), "Drop selected item".to_string()),
            ])
            .with_details(vec![
                "Inventory has limited space".to_string(),
                "Drop unnecessary items to make room".to_string(),
                "Some items stack (like arrows or potions)".to_string(),
                "Identified items show their true properties".to_string(),
            ]),
        ];
        self.entries.insert(HelpTopic::Items, items_entries);

        // 地牢探索
        let dungeon_entries = vec![
            HelpEntry::new(
                "Dungeon Structure".to_string(),
                "How the dungeon is organized".to_string(),
            )
            .with_details(vec![
                "The dungeon has multiple levels going deeper".to_string(),
                "Each level is randomly generated".to_string(),
                "Find stairs (< >) to move between levels".to_string(),
                "Deeper levels have stronger enemies and better loot".to_string(),
            ]),

            HelpEntry::new(
                "Exploration Tips".to_string(),
                "Make the most of your dungeon exploration".to_string(),
            )
            .with_details(vec![
                "Check all rooms for hidden items and secrets".to_string(),
                "Be careful around corners - enemies might be waiting".to_string(),
                "Some areas require special items to access".to_string(),
                "Remember where you've been to avoid backtracking".to_string(),
            ]),

            HelpEntry::new(
                "Environmental Hazards".to_string(),
                "Dangerous elements in the dungeon".to_string(),
            )
            .with_details(vec![
                "Traps can damage or hinder you".to_string(),
                "Some floors have special properties".to_string(),
                "Water might slow movement or conduct electricity".to_string(),
                "Fire can spread and cause burning".to_string(),
            ]),
        ];
        self.entries.insert(HelpTopic::Dungeon, dungeon_entries);

        // 角色系统
        let character_entries = vec![
            HelpEntry::new(
                "Hero Classes".to_string(),
                "Different character classes with unique abilities".to_string(),
            )
            .with_details(vec![
                "Warrior: High health and defense, good with melee weapons".to_string(),
                "Rogue: High accuracy and stealth, good for surprise attacks".to_string(),
                "Mage: Powerful spells and magic, lower physical defense".to_string(),
                "Huntress: Excellent with ranged weapons and nature magic".to_string(),
            ]),

            HelpEntry::new(
                "Character Stats".to_string(),
                "Understanding your character's attributes".to_string(),
            )
            .with_details(vec![
                "Health (HP): Your life points - don't let it reach zero!".to_string(),
                "Strength: Affects damage and equipment requirements".to_string(),
                "Accuracy: Improves chance to hit enemies".to_string(),
                "Evasion: Helps avoid enemy attacks".to_string(),
                "Defense: Reduces incoming damage".to_string(),
            ]),

            HelpEntry::new(
                "Hunger System".to_string(),
                "Managing your character's hunger".to_string(),
            )
            .with_details(vec![
                "Your character gets hungry over time".to_string(),
                "Eat food to restore hunger".to_string(),
                "Starving causes health loss".to_string(),
                "Some foods provide additional benefits".to_string(),
            ]),
        ];
        self.entries.insert(HelpTopic::Character, character_entries);

        // 游戏技巧
        let tips_entries = vec![
            HelpEntry::new(
                "Combat Tips".to_string(),
                "Strategies for effective fighting".to_string(),
            )
            .with_details(vec![
                "Use doorways to fight one enemy at a time".to_string(),
                "Attack from blind spots for stealth bonuses".to_string(),
                "Kite enemies around obstacles".to_string(),
                "Use terrain to your advantage".to_string(),
                "Don't fight when low on health - retreat and heal".to_string(),
            ]),

            HelpEntry::new(
                "Resource Management".to_string(),
                "Making the most of limited resources".to_string(),
            )
            .with_details(vec![
                "Save powerful items for tough situations".to_string(),
                "Don't waste healing items when at full health".to_string(),
                "Identify items before using them".to_string(),
                "Keep some emergency healing available".to_string(),
                "Food management is crucial for long runs".to_string(),
            ]),

            HelpEntry::new(
                "Exploration Strategy".to_string(),
                "Efficient dungeon exploration".to_string(),
            )
            .with_details(vec![
                "Clear each level thoroughly before descending".to_string(),
                "Look for secret doors and hidden areas".to_string(),
                "Remember where you left items".to_string(),
                "Plan your route to minimize backtracking".to_string(),
                "Be patient - rushing leads to mistakes".to_string(),
            ]),
        ];
        self.entries.insert(HelpTopic::Tips, tips_entries);

        // 关于游戏
        let about_entries = vec![
            HelpEntry::new(
                "Terminal Pixel Dungeon".to_string(),
                "A terminal-based roguelike adventure".to_string(),
            )
            .with_details(vec![
                "Inspired by Shattered Pixel Dungeon".to_string(),
                "Built with Rust and ratatui".to_string(),
                "Features procedural dungeon generation".to_string(),
                "Turn-based tactical combat".to_string(),
                "Permadeath - each run is unique".to_string(),
            ]),

            HelpEntry::new(
                "Game Goals".to_string(),
                "What you're trying to achieve".to_string(),
            )
            .with_details(vec![
                "Descend through the dungeon levels".to_string(),
                "Defeat enemies and collect treasure".to_string(),
                "Survive as long as possible".to_string(),
                "Discover the secrets of the dungeon".to_string(),
            ]),

            HelpEntry::new(
                "Technical Info".to_string(),
                "System information and credits".to_string(),
            )
            .with_details(vec![
                "Engine: Custom ECS with hecs".to_string(),
                "UI: ratatui + crossterm".to_string(),
                "Language: Rust 2024 Edition".to_string(),
                "Save Format: Binary (bincode)".to_string(),
                "Target: 60 FPS terminal rendering".to_string(),
            ]),
        ];
        self.entries.insert(HelpTopic::About, about_entries);
    }

    /// 获取主题的条目
    pub fn get_entries(&self, topic: &HelpTopic) -> Option<&Vec<HelpEntry>> {
        self.entries.get(topic)
    }

    /// 搜索帮助内容
    pub fn search(&self, query: &str) -> Vec<(HelpTopic, &HelpEntry)> {
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        for (topic, entries) in &self.entries {
            for entry in entries {
                if entry.title.to_lowercase().contains(&query_lower)
                    || entry.description.to_lowercase().contains(&query_lower)
                    || entry.details.iter().any(|d| d.to_lowercase().contains(&query_lower))
                {
                    results.push((topic.clone(), entry));
                }
            }
        }

        results
    }
}

/// 帮助状态
pub struct HelpState {
    /// 当前选择的主题
    current_topic: HelpTopic,
    /// 主题列表状态
    topic_state: ListState,
    /// 当前条目索引
    current_entry: usize,
    /// 条目列表状态  
    entry_state: ListState,
    /// 帮助数据库
    database: HelpDatabase,
    /// 动画管理器
    animations: AnimationManager,
    /// 是否显示搜索
    show_search: bool,
    /// 搜索文本
    search_text: String,
}

impl HelpState {
    pub fn new() -> Self {
        let mut topic_state = ListState::default();
        topic_state.select(Some(0));
        
        let mut entry_state = ListState::default();
        entry_state.select(Some(0));

        let mut animations = AnimationManager::new();
        animations.add_animation(
            "help_fade_in".to_string(),
            Animation::new(
                AnimationType::FadeIn,
                Duration::from_millis(300),
                EaseType::EaseOut,
            ),
        );

        Self {
            current_topic: HelpTopic::Controls,
            topic_state,
            current_entry: 0,
            entry_state,
            database: HelpDatabase::new(),
            animations,
            show_search: false,
            search_text: String::new(),
        }
    }

    /// 处理输入
    pub fn handle_input(&mut self, key: KeyEvent) -> bool {
        if self.show_search {
            return self.handle_search_input(key);
        }

        match key.code {
            // 主题导航
            KeyCode::Tab => {
                self.next_topic();
                true
            }
            KeyCode::BackTab => {
                self.prev_topic();
                true
            }
            
            // 条目导航
            KeyCode::Up | KeyCode::Char('k') => {
                self.prev_entry();
                true
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.next_entry();
                true
            }
            
            // 左右切换主题
            KeyCode::Left | KeyCode::Char('h') => {
                self.prev_topic();
                true
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.next_topic();
                true
            }
            
            // 搜索功能
            KeyCode::Char('/') => {
                self.show_search = true;
                self.search_text.clear();
                true
            }
            
            // 退出
            KeyCode::Esc | KeyCode::Char('q') => false,
            
            _ => true,
        }
    }

    /// 处理搜索输入
    fn handle_search_input(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char(c) => {
                self.search_text.push(c);
                true
            }
            KeyCode::Backspace => {
                self.search_text.pop();
                true
            }
            KeyCode::Enter => {
                self.show_search = false;
                // TODO: 执行搜索
                true
            }
            KeyCode::Esc => {
                self.show_search = false;
                true
            }
            _ => true,
        }
    }

    /// 下一个主题
    fn next_topic(&mut self) {
        let topics = HelpTopic::all_topics();
        let current_index = topics.iter().position(|t| *t == self.current_topic).unwrap_or(0);
        let next_index = (current_index + 1) % topics.len();
        self.current_topic = topics[next_index].clone();
        self.topic_state.select(Some(next_index));
        self.current_entry = 0;
        self.entry_state.select(Some(0));
    }

    /// 上一个主题
    fn prev_topic(&mut self) {
        let topics = HelpTopic::all_topics();
        let current_index = topics.iter().position(|t| *t == self.current_topic).unwrap_or(0);
        let prev_index = if current_index == 0 {
            topics.len() - 1
        } else {
            current_index - 1
        };
        self.current_topic = topics[prev_index].clone();
        self.topic_state.select(Some(prev_index));
        self.current_entry = 0;
        self.entry_state.select(Some(0));
    }

    /// 下一个条目
    fn next_entry(&mut self) {
        if let Some(entries) = self.database.get_entries(&self.current_topic) {
            self.current_entry = (self.current_entry + 1) % entries.len();
            self.entry_state.select(Some(self.current_entry));
        }
    }

    /// 上一个条目
    fn prev_entry(&mut self) {
        if let Some(entries) = self.database.get_entries(&self.current_topic) {
            if entries.is_empty() {
                return;
            }
            self.current_entry = if self.current_entry == 0 {
                entries.len() - 1
            } else {
                self.current_entry - 1
            };
            self.entry_state.select(Some(self.current_entry));
        }
    }

    /// 渲染帮助界面
    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        // 清空背景
        f.render_widget(Clear, area);

        // 主布局
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // 标题栏
                Constraint::Length(3), // 主题选择
                Constraint::Min(5),    // 内容区域
                Constraint::Length(1), // 状态栏
            ])
            .split(area);

        // 渲染标题
        self.render_title(f, chunks[0]);
        
        // 渲染主题选择
        self.render_topics(f, chunks[1]);
        
        // 渲染内容
        self.render_content(f, chunks[2]);
        
        // 渲染状态栏
        self.render_status(f, chunks[3]);

        // 渲染搜索框
        if self.show_search {
            self.render_search(f, area);
        }

        // 更新动画
        self.animations.update();
    }

    /// 渲染标题栏
    fn render_title(&self, f: &mut Frame, area: Rect) {
        let title = Paragraph::new("Terminal Pixel Dungeon - Help")
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        
        f.render_widget(title, area);
    }

    /// 渲染主题选择
    fn render_topics(&mut self, f: &mut Frame, area: Rect) {
        let topics = HelpTopic::all_topics();
        let tab_titles: Vec<Line> = topics
            .iter()
            .map(|topic| {
                Line::from(vec![
                    Span::styled(format!("{} ", topic.icon()), Style::default().fg(Color::Yellow)),
                    Span::styled(topic.title(), Style::default().fg(Color::White)),
                ])
            })
            .collect();

        let selected_index = topics
            .iter()
            .position(|t| *t == self.current_topic)
            .unwrap_or(0);

        let tabs = Tabs::new(tab_titles)
            .block(Block::default().borders(Borders::ALL).title(" Topics "))
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .select(selected_index);

        f.render_widget(tabs, area);
    }

    /// 渲染内容区域
    fn render_content(&mut self, f: &mut Frame, area: Rect) {
        if let Some(entries) = self.database.get_entries(&self.current_topic) {
            if entries.is_empty() {
                let empty_msg = Paragraph::new("No help available for this topic.")
                    .style(Style::default().fg(Color::Gray))
                    .alignment(Alignment::Center)
                    .block(Block::default().borders(Borders::ALL));
                f.render_widget(empty_msg, area);
                return;
            }

            let content_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
                .split(area);

            // 克隆entries以避免借用冲突
            let entries_clone = entries.clone();

            // 左侧：条目列表
            self.render_entry_list(f, content_chunks[0], &entries_clone);

            // 右侧：条目详情
            if let Some(entry) = entries_clone.get(self.current_entry) {
                self.render_entry_details(f, content_chunks[1], entry);
            }
        }
    }

    /// 渲染条目列表
    fn render_entry_list(&mut self, f: &mut Frame, area: Rect, entries: &[HelpEntry]) {
        let items: Vec<ListItem> = entries
            .iter()
            .map(|entry| ListItem::new(Line::from(entry.title.as_str())))
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" {} Entries ", self.current_topic.title()))
            )
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .bg(Color::Blue)
                    .add_modifier(Modifier::BOLD)
            );

        f.render_stateful_widget(list, area, &mut self.entry_state);
    }

    /// 渲染条目详情
    fn render_entry_details(&self, f: &mut Frame, area: Rect, entry: &HelpEntry) {
        let detail_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // 标题和描述
                Constraint::Min(3),    // 详情内容
            ])
            .split(area);

        // 标题和描述
        let title_text = vec![
            Line::from(Span::styled(
                &entry.title,
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            )),
            Line::from(Span::styled(
                &entry.description,
                Style::default().fg(Color::Gray)
            )),
        ];

        let title_paragraph = Paragraph::new(title_text)
            .block(Block::default().borders(Borders::ALL))
            .wrap(Wrap { trim: true });

        f.render_widget(title_paragraph, detail_chunks[0]);

        // 详细内容
        let mut content_lines = Vec::new();

        // 添加详情
        if !entry.details.is_empty() {
            for detail in &entry.details {
                content_lines.push(Line::from(format!("• {}", detail)));
            }
        }

        // 添加按键绑定
        if !entry.key_bindings.is_empty() {
            content_lines.push(Line::from(""));
            content_lines.push(Line::from(Span::styled(
                "Key Bindings:",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            )));
            for (key, desc) in &entry.key_bindings {
                content_lines.push(Line::from(vec![
                    Span::styled(format!("{:12}", key), Style::default().fg(Color::Green)),
                    Span::styled(desc, Style::default().fg(Color::White)),
                ]));
            }
        }

        // 添加示例
        if !entry.examples.is_empty() {
            content_lines.push(Line::from(""));
            content_lines.push(Line::from(Span::styled(
                "Examples:",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            )));
            for example in &entry.examples {
                content_lines.push(Line::from(format!("  {}", example)));
            }
        }

        let content_paragraph = Paragraph::new(content_lines)
            .block(Block::default().borders(Borders::ALL).title(" Details "))
            .wrap(Wrap { trim: true })
            .scroll((0, 0));

        f.render_widget(content_paragraph, detail_chunks[1]);
    }

    /// 渲染状态栏
    fn render_status(&self, f: &mut Frame, area: Rect) {
        let status_text = "Tab/Shift+Tab: Switch topics | ↑↓: Navigate entries | /: Search | ESC: Close";
        let status = Paragraph::new(status_text)
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);

        f.render_widget(status, area);
    }

    /// 渲染搜索框
    fn render_search(&self, f: &mut Frame, area: Rect) {
        let search_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Length(3),
                Constraint::Percentage(40),
            ])
            .split(area)[1];

        let search_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(50),
                Constraint::Percentage(25),
            ])
            .split(search_area)[1];

        f.render_widget(Clear, search_area);

        let search_widget = Paragraph::new(format!("Search: {}_", self.search_text))
            .style(Style::default().fg(Color::White))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Search Help ")
                    .style(Style::default().fg(Color::Cyan))
            );

        f.render_widget(search_widget, search_area);
    }
}

impl Default for HelpState {
    fn default() -> Self {
        Self::new()
    }
}
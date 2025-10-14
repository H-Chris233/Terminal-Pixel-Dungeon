//! 事件处理器示例
//!
//! 本文件展示如何为不同的子模块创建事件处理器

use terminal_pixel_dungeon::event_bus::{EventHandler, GameEvent, LogLevel, Priority};
use std::collections::HashMap;

// ========== Combat 模块事件处理器 ==========

/// 战斗统计处理器
/// 统计战斗相关的数据：总伤害、暴击次数、死亡数等
pub struct CombatStatisticsHandler {
    total_damage: u32,
    critical_hits: u32,
    kills: u32,
    damage_by_entity: HashMap<u32, u32>,
}

impl CombatStatisticsHandler {
    pub fn new() -> Self {
        Self {
            total_damage: 0,
            critical_hits: 0,
            kills: 0,
            damage_by_entity: HashMap::new(),
        }
    }

    pub fn get_stats(&self) -> String {
        format!(
            "总伤害: {}, 暴击: {}, 击杀: {}",
            self.total_damage, self.critical_hits, self.kills
        )
    }
}

impl EventHandler for CombatStatisticsHandler {
    fn handle(&mut self, event: &GameEvent) {
        match event {
            GameEvent::DamageDealt { attacker, damage, is_critical, .. } => {
                self.total_damage += damage;
                if *is_critical {
                    self.critical_hits += 1;
                }
                *self.damage_by_entity.entry(*attacker).or_insert(0) += damage;
            }
            GameEvent::EntityDied { .. } => {
                self.kills += 1;
            }
            _ => {}
        }
    }

    fn name(&self) -> &str {
        "CombatStatisticsHandler"
    }

    fn priority(&self) -> Priority {
        Priority::Low // 统计不需要高优先级
    }
}

// ========== Items 模块事件处理器 ==========

/// 物品使用追踪器
/// 追踪玩家使用的物品，可用于成就系统
pub struct ItemUsageTracker {
    items_used: HashMap<String, u32>,
    items_picked: u32,
    items_dropped: u32,
}

impl ItemUsageTracker {
    pub fn new() -> Self {
        Self {
            items_used: HashMap::new(),
            items_picked: 0,
            items_dropped: 0,
        }
    }

    pub fn get_usage_count(&self, item_name: &str) -> u32 {
        *self.items_used.get(item_name).unwrap_or(&0)
    }

    pub fn most_used_item(&self) -> Option<(&String, &u32)> {
        self.items_used.iter().max_by_key(|(_, count)| *count)
    }
}

impl EventHandler for ItemUsageTracker {
    fn handle(&mut self, event: &GameEvent) {
        match event {
            GameEvent::ItemUsed { item_name, .. } => {
                *self.items_used.entry(item_name.clone()).or_insert(0) += 1;
            }
            GameEvent::ItemPickedUp { .. } => {
                self.items_picked += 1;
            }
            GameEvent::ItemDropped { .. } => {
                self.items_dropped += 1;
            }
            _ => {}
        }
    }

    fn name(&self) -> &str {
        "ItemUsageTracker"
    }

    fn priority(&self) -> Priority {
        Priority::Normal
    }
}

// ========== Dungeon 模块事件处理器 ==========

/// 地牢探索追踪器
/// 追踪玩家的探索进度和发现
pub struct DungeonExplorationTracker {
    current_level: usize,
    max_level_reached: usize,
    rooms_discovered: Vec<usize>,
    traps_triggered: u32,
}

impl DungeonExplorationTracker {
    pub fn new() -> Self {
        Self {
            current_level: 1,
            max_level_reached: 1,
            rooms_discovered: Vec::new(),
            traps_triggered: 0,
        }
    }

    pub fn exploration_percentage(&self) -> f32 {
        // 简化计算：假设每层有10个房间
        let total_rooms = self.current_level * 10;
        (self.rooms_discovered.len() as f32 / total_rooms as f32) * 100.0
    }
}

impl EventHandler for DungeonExplorationTracker {
    fn handle(&mut self, event: &GameEvent) {
        match event {
            GameEvent::LevelChanged { new_level, .. } => {
                self.current_level = *new_level;
                if *new_level > self.max_level_reached {
                    self.max_level_reached = *new_level;
                }
            }
            GameEvent::RoomDiscovered { room_id } => {
                if !self.rooms_discovered.contains(room_id) {
                    self.rooms_discovered.push(*room_id);
                }
            }
            GameEvent::TrapTriggered { .. } => {
                self.traps_triggered += 1;
            }
            _ => {}
        }
    }

    fn name(&self) -> &str {
        "DungeonExplorationTracker"
    }

    fn priority(&self) -> Priority {
        Priority::Normal
    }
}

// ========== Achievement 系统事件处理器 ==========

/// 成就解锁器
/// 根据游戏事件自动解锁成就
pub struct AchievementUnlocker {
    achievements: Vec<String>,
}

impl AchievementUnlocker {
    pub fn new() -> Self {
        Self {
            achievements: Vec::new(),
        }
    }

    fn unlock(&mut self, achievement: &str) {
        if !self.achievements.contains(&achievement.to_string()) {
            self.achievements.push(achievement.to_string());
            println!("🏆 成就解锁: {}", achievement);
        }
    }

    pub fn has_achievement(&self, achievement: &str) -> bool {
        self.achievements.contains(&achievement.to_string())
    }
}

impl EventHandler for AchievementUnlocker {
    fn handle(&mut self, event: &GameEvent) {
        match event {
            GameEvent::EntityDied { entity_name, .. } => {
                // 首次击杀
                if !self.has_achievement("初次胜利") {
                    self.unlock("初次胜利");
                }

                // Boss 击杀
                if entity_name.contains("Boss") || entity_name.contains("首领") {
                    self.unlock("屠龙勇士");
                }
            }
            GameEvent::DamageDealt { damage, is_critical, .. } => {
                // 高伤害暴击
                if *is_critical && *damage >= 100 {
                    self.unlock("毁灭打击");
                }
            }
            GameEvent::Victory => {
                self.unlock("胜利者");
            }
            GameEvent::LevelChanged { new_level, .. } => {
                // 深入地牢
                if *new_level >= 10 {
                    self.unlock("地牢探险家");
                }
                if *new_level >= 25 {
                    self.unlock("深渊行者");
                }
            }
            _ => {}
        }
    }

    fn name(&self) -> &str {
        "AchievementUnlocker"
    }

    fn priority(&self) -> Priority {
        Priority::Low
    }
}

// ========== 调试辅助处理器 ==========

/// 性能监控处理器
/// 监控事件处理的性能指标
pub struct PerformanceMonitor {
    event_counts: HashMap<&'static str, usize>,
    start_time: std::time::Instant,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            event_counts: HashMap::new(),
            start_time: std::time::Instant::now(),
        }
    }

    pub fn events_per_second(&self) -> f64 {
        let total_events: usize = self.event_counts.values().sum();
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            total_events as f64 / elapsed
        } else {
            0.0
        }
    }

    pub fn print_stats(&self) {
        println!("=== 事件性能统计 ===");
        println!("运行时间: {:.2}s", self.start_time.elapsed().as_secs_f64());
        println!("事件/秒: {:.2}", self.events_per_second());
        println!("\n事件类型统计:");
        let mut sorted: Vec<_> = self.event_counts.iter().collect();
        sorted.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
        for (event_type, count) in sorted.iter().take(10) {
            println!("  {}: {}", event_type, count);
        }
    }
}

impl EventHandler for PerformanceMonitor {
    fn handle(&mut self, event: &GameEvent) {
        let event_type = event.event_type();
        *self.event_counts.entry(event_type).or_insert(0) += 1;
    }

    fn name(&self) -> &str {
        "PerformanceMonitor"
    }

    fn priority(&self) -> Priority {
        Priority::Lowest // 性能监控应该是最低优先级
    }
}

/// 事件调试器
/// 打印详细的事件信息用于调试
pub struct EventDebugger {
    verbose: bool,
    filter: Vec<&'static str>,
}

impl EventDebugger {
    pub fn new(verbose: bool) -> Self {
        Self {
            verbose,
            filter: Vec::new(),
        }
    }

    pub fn with_filter(mut self, event_types: Vec<&'static str>) -> Self {
        self.filter = event_types;
        self
    }
}

impl EventHandler for EventDebugger {
    fn handle(&mut self, event: &GameEvent) {
        if self.verbose {
            println!("[DEBUG] {:?}", event);
        } else {
            println!("[DEBUG] {}", event.event_type());
        }
    }

    fn name(&self) -> &str {
        "EventDebugger"
    }

    fn priority(&self) -> Priority {
        Priority::Lowest
    }

    fn should_handle(&self, event: &GameEvent) -> bool {
        if self.filter.is_empty() {
            true
        } else {
            self.filter.contains(&event.event_type())
        }
    }
}

// ========== 使用示例 ==========

#[cfg(test)]
mod examples {
    use super::*;
    use terminal_pixel_dungeon::event_bus::EventBus;

    #[test]
    fn example_combat_statistics() {
        let mut event_bus = EventBus::new();
        let stats_handler = CombatStatisticsHandler::new();

        // 注册处理器
        event_bus.subscribe_all(Box::new(stats_handler));

        // 模拟战斗事件
        event_bus.publish(GameEvent::DamageDealt {
            attacker: 1,
            victim: 2,
            damage: 50,
            is_critical: true,
        });

        event_bus.publish(GameEvent::DamageDealt {
            attacker: 1,
            victim: 2,
            damage: 30,
            is_critical: false,
        });

        event_bus.publish(GameEvent::EntityDied {
            entity: 2,
            entity_name: "哥布林".to_string(),
        });

        // 统计会自动更新
        // 在实际应用中，你可以获取处理器的引用来查询统计数据
    }

    #[test]
    fn example_achievement_system() {
        let mut event_bus = EventBus::new();
        let achievement_handler = AchievementUnlocker::new();

        event_bus.subscribe_all(Box::new(achievement_handler));

        // 触发成就
        event_bus.publish(GameEvent::EntityDied {
            entity: 100,
            entity_name: "哥布林".to_string(),
        });

        event_bus.publish(GameEvent::DamageDealt {
            attacker: 1,
            victim: 2,
            damage: 150,
            is_critical: true,
        });

        event_bus.publish(GameEvent::Victory);
    }

    #[test]
    fn example_performance_monitoring() {
        let mut event_bus = EventBus::new();
        let mut perf_monitor = PerformanceMonitor::new();

        event_bus.subscribe_all(Box::new(PerformanceMonitor::new()));

        // 模拟大量事件
        for _ in 0..1000 {
            event_bus.publish(GameEvent::PlayerTurnStarted);
        }

        // 在实际应用中打印统计
        // perf_monitor.print_stats();
    }

    #[test]
    fn example_debug_filtered() {
        let mut event_bus = EventBus::new();

        // 只调试战斗相关事件
        let debugger = EventDebugger::new(true)
            .with_filter(vec!["DamageDealt", "EntityDied", "CombatStarted"]);

        event_bus.subscribe_all(Box::new(debugger));

        // 这些会被打印
        event_bus.publish(GameEvent::DamageDealt {
            attacker: 1,
            victim: 2,
            damage: 10,
            is_critical: false,
        });

        // 这个不会被打印（被过滤了）
        event_bus.publish(GameEvent::PlayerTurnStarted);
    }

    #[test]
    fn example_multiple_handlers() {
        let mut event_bus = EventBus::new();

        // 同时注册多个处理器
        event_bus.subscribe_all(Box::new(CombatStatisticsHandler::new()));
        event_bus.subscribe_all(Box::new(ItemUsageTracker::new()));
        event_bus.subscribe_all(Box::new(AchievementUnlocker::new()));
        event_bus.subscribe_all(Box::new(PerformanceMonitor::new()));

        // 一个事件会被所有处理器处理
        event_bus.publish(GameEvent::DamageDealt {
            attacker: 1,
            victim: 2,
            damage: 50,
            is_critical: true,
        });
    }
}

// ========== 快速开始指南 ==========

/// 创建自定义事件处理器的步骤：
///
/// 1. 定义处理器结构体
/// ```rust
/// pub struct MyHandler {
///     // 你的状态字段
///     counter: u32,
/// }
/// ```
///
/// 2. 实现 EventHandler trait
/// ```rust
/// impl EventHandler for MyHandler {
///     fn handle(&mut self, event: &GameEvent) {
///         match event {
///             GameEvent::DamageDealt { damage, .. } => {
///                 self.counter += damage;
///             }
///             _ => {}
///         }
///     }
///
///     fn name(&self) -> &str { "MyHandler" }
/// }
/// ```
///
/// 3. 注册到事件总线
/// ```rust
/// let mut event_bus = EventBus::new();
/// event_bus.subscribe_all(Box::new(MyHandler { counter: 0 }));
/// ```
///
/// 4. 发布事件
/// ```rust
/// event_bus.publish(GameEvent::DamageDealt { ... });
/// ```

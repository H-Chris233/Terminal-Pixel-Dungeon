// src/hero/effects.rs
use bincode::{Decode, Encode};
pub use combat::effect::{Effect, EffectType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::core::Hero;
use crate::dungeon;

use combat::Combatant;

/// 英雄效果管理系统
#[derive(Clone, Debug, Default, Serialize, Deserialize, Encode, Decode)]
pub struct EffectManager {
    #[bincode(with_serde)]
    effects: HashMap<EffectType, Effect>,
}

impl EffectManager {
    /// 创建空的效果管理器
    pub fn new() -> Self {
        Self {
            effects: HashMap::new(),
        }
    }

    /// 添加或更新效果（带互斥检查）
    pub fn add(&mut self, effect: Effect) -> bool {
        if self.has_conflicting_effect(effect.effect_type()) {
            return false;
        }

        self.effects
            .entry(effect.effect_type())
            .and_modify(|e| {
                if e.is_overwritable() {
                    // 覆盖规则：取最大持续时间和强度
                    *e = Effect::with_intensity(
                        effect.effect_type(),
                        e.turns().max(effect.turns()),
                        e.intensity().max(effect.intensity()),
                    );
                } else if e.is_stackable() {
                    // 叠加规则：持续时间相加，强度取最大值
                    *e = Effect::with_intensity(
                        effect.effect_type(),
                        e.turns() + effect.turns(),
                        e.intensity().max(effect.intensity()),
                    );
                }
            })
            .or_insert(effect);

        true
    }

    /// 检查效果互斥性
    fn has_conflicting_effect(&self, new_effect: EffectType) -> bool {
        self.effects.keys().any(|&existing| {
            matches!(
                (existing, new_effect),
                (EffectType::Burning, EffectType::Frost)
                    | (EffectType::Frost, EffectType::Burning)
                    | (EffectType::Haste, EffectType::Slow)
                    | (EffectType::Slow, EffectType::Haste)
                    | (EffectType::Invisibility, EffectType::Light)
                    | (EffectType::Light, EffectType::Invisibility)
            )
        })
    }

    /// 移除指定类型的效果
    pub fn remove(&mut self, effect_type: EffectType) -> Option<Effect> {
        self.effects.remove(&effect_type)
    }

    /// 强制添加效果（忽略互斥规则）
    pub fn add_force(&mut self, effect: Effect) {
        self.effects.insert(effect.effect_type(), effect);
    }

    /// 检查是否存在指定效果
    pub fn has(&self, effect_type: EffectType) -> bool {
        self.effects.contains_key(&effect_type)
    }

    /// 获取效果持续时间（回合数）
    pub fn get_turns(&self, effect_type: EffectType) -> u32 {
        self.effects.get(&effect_type).map_or(0, |e| e.turns())
    }

    /// 更新所有效果（每回合调用）
    pub fn update(&mut self) -> Vec<EffectType> {
        let mut expired = Vec::new();

        self.effects.retain(|&ty, e| {
            let keep = e.update();
            if !keep {
                expired.push(ty);
            }
            keep
        });

        expired
    }

    /// 清除所有效果
    pub fn clear(&mut self) {
        self.effects.clear();
    }

    /// 获取当前生效的所有效果
    pub fn active_effects(&self) -> Vec<&Effect> {
        self.effects.values().collect()
    }

    /// 检查移动限制效果
    pub fn is_immobilized(&self) -> bool {
        self.has(EffectType::Paralysis) || self.has(EffectType::Rooted)
    }

    /// 检查视觉增强效果
    pub fn has_vision_enhancement(&self) -> bool {
        self.has(EffectType::MindVision)
    }

    /// 延长指定效果的持续时间
    pub fn extend_duration(&mut self, effect_type: EffectType, extra_turns: u32) -> bool {
        if let Some(effect) = self.effects.get_mut(&effect_type) {
            effect.set_turns(effect.turns() + extra_turns);
            true
        } else {
            false
        }
    }
}

/// 为Hero实现效果处理
impl Hero {
    /// 处理效果伤害（每回合调用）
    pub fn process_effects(&mut self) {
        let mut damage_total = 0;

        // 计算所有伤害型效果
        for effect in self.effects.active_effects() {
            damage_total += effect.damage();
        }

        // 应用总伤害
        if damage_total > 0 {
            self.take_damage(damage_total);
        }
    }
}

/// 效果系统trait实现
pub trait EffectSystem {
    fn add_effect(&mut self, effect: Effect);
    fn remove_effect(&mut self, effect_type: EffectType);
    fn has_effect(&self, effect_type: EffectType) -> bool;
    fn update_effects(&mut self);
}

impl EffectSystem for Hero {
    fn add_effect(&mut self, effect: Effect) {
        // 优先处理可叠加效果
        if effect.is_stackable() && self.effects.has(effect.effect_type()) {
            if let Some(existing) = self.effects.effects.get_mut(&effect.effect_type()) {
                *existing = Effect::with_intensity(
                    effect.effect_type(),
                    existing.turns() + effect.turns(),
                    existing.intensity().max(effect.intensity()),
                );
                return;
            }
        }

        self.effects.add(effect);
    }

    fn remove_effect(&mut self, effect_type: EffectType) {
        self.effects.remove(effect_type);

        // 特殊效果移除处理
        match effect_type {
            EffectType::Invisibility => {
                dungeon::alert_nearby_enemies(self.x, self.y);
            }
            _ => {}
        }
    }

    fn has_effect(&self, effect_type: EffectType) -> bool {
        self.effects.has(effect_type)
    }

    fn update_effects(&mut self) {
        let expired = self.effects.update();

        // 处理过期效果的特殊逻辑
        for effect_type in expired {
            match effect_type {
                EffectType::Invisibility => {
                    dungeon::alert_nearby_enemies(self.x, self.y);
                }
                _ => {}
            }
        }

        // 处理本回合伤害
        self.process_effects();
    }
}


// src/hero/effects.rs
use bincode::{Decode, Encode};
pub use combat::effect::{Effect, EffectType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::EffectSystem;
use crate::core::Hero;

/// 效果管理系统
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
                    *e = Effect::new(
                        effect.effect_type(),
                        e.turns().max(effect.turns()),
                        effect.damage().max(e.damage()),
                        effect.damage_interval(),
                    );
                } else if e.is_stackable() {
                    *e = Effect::new(
                        effect.effect_type(),
                        e.turns() + effect.turns(),
                        e.damage().max(effect.damage()),
                        e.damage_interval().min(effect.damage_interval()),
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
                (EffectType::Burning, EffectType::Frozen) |
                (EffectType::Frozen, EffectType::Burning) |
                (EffectType::Haste, EffectType::Slow) |
                (EffectType::Slow, EffectType::Haste) |
                (EffectType::Invisible, EffectType::Revealed) |
                (EffectType::Revealed, EffectType::Invisible)
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
    pub fn update(&mut self) -> Vec<(EffectType, u32)> {
        let mut expired = Vec::new();

        self.effects.retain(|&ty, e| {
            let new_turns = e.turns().saturating_sub(1);
            if new_turns == 0 {
                expired.push((ty, e.damage().unwrap_or(0)));
                false
            } else {
                *e = Effect::new(ty, new_turns, e.damage(), e.damage_interval());
                true
            }
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
        self.has(EffectType::MindVision) || self.has(EffectType::DarkVision)
    }

    /// 获取效果伤害值
    pub fn get_damage(&self, effect_type: EffectType) -> Option<u32> {
        self.effects.get(&effect_type).and_then(|e| e.damage())
    }

    /// 获取所有伤害型效果
    pub fn damaging_effects(&self) -> Vec<&Effect> {
        self.effects
            .values()
            .filter(|e| e.damage().is_some())
            .collect()
    }

    /// 延长指定效果的持续时间
    pub fn extend_duration(&mut self, effect_type: EffectType, extra_turns: u32) -> bool {
        if let Some(effect) = self.effects.get_mut(&effect_type) {
            *effect = Effect::new(
                effect_type,
                effect.turns() + extra_turns,
                effect.damage(),
                effect.damage_interval(),
            );
            true
        } else {
            false
        }
    }
}


impl EffectSystem for Hero {
    /// 添加新效果到英雄身上
    fn add(&mut self, effect: Effect) {
        // 检查效果冲突（例如：不能同时有多个同类型效果）
        if self.effects.has(effect.effect_type) {
            // 已有同类型效果时的处理策略：
            // 1. 叠加持续时间（如中毒）
            // 2. 替换更强效果（如增益效果）
            // 3. 忽略新效果（如免疫）
            match effect.effect_type {
                EffectType::Poison | EffectType::Regeneration => {
                    // 可叠加效果：延长持续时间
                    if let Some(existing) = self.effects.get_mut(effect.effect_type) {
                        existing.duration += effect.duration;
                    }
                }
                _ => {
                    // 默认行为：替换现有效果
                    self.effects.remove(effect.effect_type);
                    self.effects.add(effect);
                }
            }
        } else {
            // 无冲突直接添加
            self.effects.add(effect);
        }
    }

    /// 移除指定类型的效果
    fn remove(&mut self, effect_type: EffectType) {
        self.effects.remove(effect_type);
        
        // 移除后的额外处理
        match effect_type {
            EffectType::Invisibility => {
                // 显形时可能需要通知周围敌人
                dungeon::alert_nearby_enemies(self.x, self.y);
            }
            _ => {}
        }
    }

    /// 检查是否具有某种效果
    fn has(&self, effect_type: EffectType) -> bool {
        self.effects.has(effect_type)
    }

    /// 每回合更新所有效果状态
    fn update(&mut self) {
        // 先更新所有效果
        self.effects.update(self);
        
        // 效果更新后的处理
        if self.effects.any_expired() {
            // 可以在这里添加效果结束的通知逻辑
            // 例如：中毒结束、增益消失等
        }
        
        // 特殊效果处理
        if self.has(EffectType::Poison) {
            if let Some(poison) = self.effects.get(EffectType::Poison) {
                self.take_damage(poison.power);
            }
        }
        
        if self.has(EffectType::Regeneration) {
            if let Some(regen) = self.effects.get(EffectType::Regeneration) {
                self.heal(regen.power);
            }
        }
    }
}


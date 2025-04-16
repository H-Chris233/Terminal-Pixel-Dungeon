
// src/hero/effects.rs
use bincode::{Decode, Encode};
use combat::effect::{Effect, EffectType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;


/// 效果管理系统
#[derive(Clone, Debug, Default, Serialize, Deserialize, Encode, Decode)]
pub struct EffectManager {
    #[bincode(with_serde)]
    effects: HashMap<EffectType, Effect>,
}

impl EffectManager {
    pub fn new() -> Self {
        Self {
            effects: HashMap::new(),
        }
    }

    /// 添加或更新效果（带互斥检查）
    pub fn add(&mut self, effect: Effect) -> bool {
        // 检查互斥效果
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
    pub fn has_conflicting_effect(&self, new_effect: EffectType) -> bool {
        self.effects.keys().any(|&existing| {
            matches!(
                (existing, new_effect),
                // 燃烧与冰冻互斥
                (EffectType::Burning, EffectType::Frozen) |
                (EffectType::Frozen, EffectType::Burning) |
                // 加速与减速互斥
                (EffectType::Haste, EffectType::Slow) |
                (EffectType::Slow, EffectType::Haste) |
                // 隐身与显形互斥
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
        self.effects
            .get(&effect_type)
            .map_or(0, |e| e.turns())
    }

    /// 更新所有效果（每回合调用）
    pub fn update(&mut self) -> Vec<(EffectType, u32)> {
        let mut expired = Vec::new();

        // 减少持续时间并收集过期效果
        self.effects.retain(|&ty, e| {
            let new_turns = e.turns().saturating_sub(1);
            if new_turns == 0 {
                expired.push((ty, e.damage()));
                false
            } else {
                *e = Effect::new(
                    ty,
                    new_turns,
                    e.damage(),
                    e.damage_interval(),
                );
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

    /// 检查是否有任何限制移动的效果
    pub fn is_immobilized(&self) -> bool {
        self.has(EffectType::Paralysis) || self.has(EffectType::Rooted)
    }

    /// 检查是否有视觉增强效果
    pub fn has_vision_enhancement(&self) -> bool {
        self.has(EffectType::MindVision)
    }

    /// 获取效果伤害（如果有）
    pub fn get_damage(&self, effect_type: EffectType) -> Option<u32> {
        self.effects
            .get(&effect_type)
            .and_then(|e| e.damage())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use combat::effect::Effect;

    #[test]
    fn test_effect_management() {
        let mut manager = EffectManager::new();
        
        // 测试添加效果
        let poison = Effect::new(EffectType::Poison, 5, Some(2), 1);
        manager.add(poison);
        assert!(manager.has(EffectType::Poison));
        assert_eq!(manager.get_turns(EffectType::Poison), 5);
        
        // 测试效果叠加
        let more_poison = Effect::new(EffectType::Poison, 3, Some(3), 1);
        manager.add(more_poison);
        assert_eq!(manager.get_turns(EffectType::Poison), 8);
        
        // 测试效果更新
        let expired = manager.update();
        assert!(expired.is_empty());
        assert_eq!(manager.get_turns(EffectType::Poison), 7);
        
        // 测试效果移除
        manager.remove(EffectType::Poison);
        assert!(!manager.has(EffectType::Poison));
    }

    #[test]
    fn test_effect_expiration() {
        let mut manager = EffectManager::new();
        manager.add(Effect::new(EffectType::Burning, 1, Some(3), 1));
        
        let expired = manager.update();
        assert_eq!(expired.len(), 1);
        assert_eq!(expired[0].0, EffectType::Burning);
        assert!(!manager.has(EffectType::Burning));
    }
    #[test]
    fn test_effect_conflicts() {
        let mut manager = EffectManager::new();
        
        // 添加燃烧效果
        assert!(manager.add(Effect::new(EffectType::Burning, 5, Some(2), 1)));
        
        // 尝试添加冰冻（应该失败）
        assert!(!manager.add(Effect::new(EffectType::Frozen, 3, None, 0)));
        
        // 强制添加冰冻
        manager.add_force(Effect::new(EffectType::Frozen, 3, None, 0));
        assert!(manager.has(EffectType::Burning));
        assert!(manager.has(EffectType::Frozen));
        
        // 移除燃烧后可以正常添加冰冻
        manager.remove(EffectType::Burning);
        assert!(manager.add(Effect::new(EffectType::Frozen, 3, None, 0)));
    }
    
}

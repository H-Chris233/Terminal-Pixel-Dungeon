use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::class::Class;

/// Trait for class abilities (active skills and passive perks)
pub trait ClassAbility {
    /// Get the ability name
    fn name(&self) -> &str;
    
    /// Get the ability description
    fn description(&self) -> &str;
    
    /// Get the energy cost to activate (0 for passive abilities)
    fn energy_cost(&self) -> u32;
    
    /// Get the cooldown in turns (0 for no cooldown)
    fn cooldown(&self) -> u32;
    
    /// Check if this is a passive ability
    fn is_passive(&self) -> bool;
}

/// Active skill effect type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
pub enum SkillEffect {
    /// Deal direct damage
    Damage { amount: u32, ignore_armor: bool },
    /// Heal the user
    Heal { amount: u32 },
    /// Apply a status effect
    StatusEffect { effect_type: combat::effect::EffectType, duration: u32 },
    /// Guaranteed critical hit on next attack
    GuaranteedCrit,
    /// Teleport/dash to target
    Dash { range: u8 },
    /// Shield that absorbs damage
    Shield { amount: u32, duration: u32 },
    /// Modify stats temporarily (uses integer percentages instead of floats)
    StatBuff { attack_percent: i8, defense_percent: i8, duration: u32 },
}

/// Active skill definition
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
pub struct ActiveSkill {
    pub name: String,
    pub description: String,
    pub energy_cost: u32,
    pub cooldown: u32,
    pub effects: Vec<SkillEffect>,
    pub range: u8,
    pub requires_target: bool,
}

impl ClassAbility for ActiveSkill {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn energy_cost(&self) -> u32 {
        self.energy_cost
    }
    
    fn cooldown(&self) -> u32 {
        self.cooldown
    }
    
    fn is_passive(&self) -> bool {
        false
    }
}

/// Passive perk trigger condition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
pub enum PassiveTrigger {
    /// Trigger every turn
    PerTurn,
    /// Trigger when HP is below a threshold (percentage)
    LowHealth { threshold: u8 },
    /// Trigger when attacking
    OnAttack,
    /// Trigger when being attacked
    OnDefend,
    /// Trigger when dealing a critical hit
    OnCrit,
    /// Trigger when using a skill
    OnSkillUse,
}

/// Passive perk definition
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
pub struct PassivePerk {
    pub name: String,
    pub description: String,
    pub trigger: PassiveTrigger,
    pub effect: PerkEffect,
}

/// Passive perk effect type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
pub enum PerkEffect {
    /// Regenerate HP each turn
    Regeneration { amount: u32 },
    /// Increase damage dealt
    DamageBonus { percent: u8 },
    /// Increase critical hit chance
    CritBonus { percent: u8 },
    /// Increase dodge chance
    DodgeBonus { percent: u8 },
    /// Reduce damage taken
    DamageReduction { percent: u8 },
    /// Grant temporary buff when triggered (uses integer percentages)
    TriggerBuff { attack_percent: i8, defense_percent: i8, duration: u32 },
}

impl ClassAbility for PassivePerk {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn energy_cost(&self) -> u32 {
        0
    }
    
    fn cooldown(&self) -> u32 {
        0
    }
    
    fn is_passive(&self) -> bool {
        true
    }
}

/// Class ability set for each class
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
pub struct ClassAbilitySet {
    pub active_skills: Vec<ActiveSkill>,
    pub passive_perks: Vec<PassivePerk>,
}

impl ClassAbilitySet {
    pub fn for_class(class: Class) -> Self {
        match class {
            Class::Warrior => Self::warrior_abilities(),
            Class::Mage => Self::mage_abilities(),
            Class::Rogue => Self::rogue_abilities(),
            Class::Huntress => Self::huntress_abilities(),
        }
    }
    
    fn warrior_abilities() -> Self {
        Self {
            active_skills: vec![
                ActiveSkill {
                    name: "英勇冲锋".to_string(),
                    description: "冲向目标并造成伤害，击晕目标1回合".to_string(),
                    energy_cost: 100,
                    cooldown: 5,
                    effects: vec![
                        SkillEffect::Dash { range: 3 },
                        SkillEffect::Damage { amount: 20, ignore_armor: false },
                        SkillEffect::StatusEffect { 
                            effect_type: combat::effect::EffectType::Paralysis, 
                            duration: 1 
                        },
                    ],
                    range: 3,
                    requires_target: true,
                },
            ],
            passive_perks: vec![
                PassivePerk {
                    name: "战斗再生".to_string(),
                    description: "当生命值低于50%时，每回合恢复2点生命值".to_string(),
                    trigger: PassiveTrigger::LowHealth { threshold: 50 },
                    effect: PerkEffect::Regeneration { amount: 2 },
                },
            ],
        }
    }
    
    fn mage_abilities() -> Self {
        Self {
            active_skills: vec![
                ActiveSkill {
                    name: "奥术冲击".to_string(),
                    description: "释放魔法能量，造成无视护甲的伤害".to_string(),
                    energy_cost: 100,
                    cooldown: 4,
                    effects: vec![
                        SkillEffect::Damage { amount: 25, ignore_armor: true },
                    ],
                    range: 5,
                    requires_target: true,
                },
            ],
            passive_perks: vec![
                PassivePerk {
                    name: "法力护盾".to_string(),
                    description: "受到伤害时减少15%的伤害".to_string(),
                    trigger: PassiveTrigger::OnDefend,
                    effect: PerkEffect::DamageReduction { percent: 15 },
                },
            ],
        }
    }
    
    fn rogue_abilities() -> Self {
        Self {
            active_skills: vec![
                ActiveSkill {
                    name: "影袭".to_string(),
                    description: "下次攻击必定暴击".to_string(),
                    energy_cost: 100,
                    cooldown: 6,
                    effects: vec![
                        SkillEffect::GuaranteedCrit,
                    ],
                    range: 0,
                    requires_target: false,
                },
            ],
            passive_perks: vec![
                PassivePerk {
                    name: "奇袭".to_string(),
                    description: "从隐身状态攻击时额外造成25%伤害".to_string(),
                    trigger: PassiveTrigger::OnAttack,
                    effect: PerkEffect::DamageBonus { percent: 25 },
                },
            ],
        }
    }
    
    fn huntress_abilities() -> Self {
        Self {
            active_skills: vec![
                ActiveSkill {
                    name: "精准射击".to_string(),
                    description: "快速射击，造成伤害并减少目标闪避".to_string(),
                    energy_cost: 80,
                    cooldown: 3,
                    effects: vec![
                        SkillEffect::Damage { amount: 15, ignore_armor: false },
                        SkillEffect::StatBuff { 
                            attack_percent: 0, 
                            defense_percent: -30, 
                            duration: 2 
                        },
                    ],
                    range: 6,
                    requires_target: true,
                },
            ],
            passive_perks: vec![
                PassivePerk {
                    name: "自然守护".to_string(),
                    description: "增加10%闪避几率".to_string(),
                    trigger: PassiveTrigger::OnDefend,
                    effect: PerkEffect::DodgeBonus { percent: 10 },
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_warrior_abilities() {
        let abilities = ClassAbilitySet::for_class(Class::Warrior);
        assert_eq!(abilities.active_skills.len(), 1);
        assert_eq!(abilities.passive_perks.len(), 1);
        
        let skill = &abilities.active_skills[0];
        assert_eq!(skill.name(), "英勇冲锋");
        assert_eq!(skill.energy_cost(), 100);
        assert_eq!(skill.cooldown(), 5);
        assert!(!skill.is_passive());
        
        let perk = &abilities.passive_perks[0];
        assert_eq!(perk.name(), "战斗再生");
        assert_eq!(perk.energy_cost(), 0);
        assert!(perk.is_passive());
    }

    #[test]
    fn test_all_classes_have_abilities() {
        for class in [Class::Warrior, Class::Mage, Class::Rogue, Class::Huntress] {
            let abilities = ClassAbilitySet::for_class(class.clone());
            assert!(!abilities.active_skills.is_empty(), "{:?} should have active skills", class);
            assert!(!abilities.passive_perks.is_empty(), "{:?} should have passive perks", class);
        }
    }
}

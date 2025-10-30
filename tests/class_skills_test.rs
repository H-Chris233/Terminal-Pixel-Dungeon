use hero::{Hero, abilities::{ClassAbilitySet, ClassAbility}, class::{Class, SkillState}};

#[test]
fn test_skill_state_cooldown_tracking() {
    let mut skill_state = SkillState::new();
    
    // Set cooldown for a skill
    skill_state.set_cooldown("英勇冲锋".to_string(), 5);
    assert_eq!(skill_state.get_cooldown("英勇冲锋"), 5);
    assert!(!skill_state.is_skill_ready("英勇冲锋"));
    
    // Tick cooldown
    skill_state.tick_cooldowns();
    assert_eq!(skill_state.get_cooldown("英勇冲锋"), 4);
    
    // Tick until ready
    for _ in 0..4 {
        skill_state.tick_cooldowns();
    }
    assert_eq!(skill_state.get_cooldown("英勇冲锋"), 0);
    assert!(skill_state.is_skill_ready("英勇冲锋"));
}

#[test]
fn test_skill_state_charges() {
    let mut skill_state = SkillState::new();
    
    // Set charges
    skill_state.set_charges("魔法飞弹".to_string(), 3);
    assert_eq!(skill_state.get_charges("魔法飞弹"), 3);
    
    // Consume charge
    assert!(skill_state.consume_charge("魔法飞弹"));
    assert_eq!(skill_state.get_charges("魔法飞弹"), 2);
    
    // Add charges
    skill_state.add_charge("魔法飞弹".to_string(), 2);
    assert_eq!(skill_state.get_charges("魔法飞弹"), 4);
}

#[test]
fn test_warrior_abilities() {
    let abilities = ClassAbilitySet::for_class(Class::Warrior);
    
    // Check active skill
    assert_eq!(abilities.active_skills.len(), 1);
    let skill = &abilities.active_skills[0];
    assert_eq!(skill.name(), "英勇冲锋");
    assert_eq!(skill.energy_cost(), 100);
    assert_eq!(skill.cooldown(), 5);
    assert!(!skill.is_passive());
    
    // Check passive perk
    assert_eq!(abilities.passive_perks.len(), 1);
    let perk = &abilities.passive_perks[0];
    assert_eq!(perk.name(), "战斗再生");
    assert_eq!(perk.energy_cost(), 0);
    assert!(perk.is_passive());
}

#[test]
fn test_mage_abilities() {
    let abilities = ClassAbilitySet::for_class(Class::Mage);
    
    assert_eq!(abilities.active_skills.len(), 1);
    let skill = &abilities.active_skills[0];
    assert_eq!(skill.name(), "奥术冲击");
    assert_eq!(skill.cooldown(), 4);
    
    assert_eq!(abilities.passive_perks.len(), 1);
    let perk = &abilities.passive_perks[0];
    assert_eq!(perk.name(), "法力护盾");
}

#[test]
fn test_rogue_abilities() {
    let abilities = ClassAbilitySet::for_class(Class::Rogue);
    
    assert_eq!(abilities.active_skills.len(), 1);
    let skill = &abilities.active_skills[0];
    assert_eq!(skill.name(), "影袭");
    assert_eq!(skill.cooldown(), 6);
    
    assert_eq!(abilities.passive_perks.len(), 1);
    let perk = &abilities.passive_perks[0];
    assert_eq!(perk.name(), "奇袭");
}

#[test]
fn test_huntress_abilities() {
    let abilities = ClassAbilitySet::for_class(Class::Huntress);
    
    assert_eq!(abilities.active_skills.len(), 1);
    let skill = &abilities.active_skills[0];
    assert_eq!(skill.name(), "精准射击");
    assert_eq!(skill.cooldown(), 3);
    
    assert_eq!(abilities.passive_perks.len(), 1);
    let perk = &abilities.passive_perks[0];
    assert_eq!(perk.name(), "自然守护");
}

#[test]
fn test_hero_skill_cooldown_ticking() {
    let mut hero = Hero::new(Class::Warrior);
    
    // Use a skill (simulate)
    hero.class_skills.set_cooldown("英勇冲锋".to_string(), 5);
    
    // Advance turns and check cooldown decreases
    for i in (1..=5).rev() {
        assert_eq!(hero.class_skills.get_cooldown("英勇冲锋"), i);
        let _ = hero.on_turn();
    }
    
    // After 5 turns, skill should be ready
    assert!(hero.class_skills.is_skill_ready("英勇冲锋"));
}

#[test]
fn test_hero_try_use_skill() {
    let mut hero = Hero::new(Class::Warrior);
    
    // First use should succeed
    assert!(hero.try_use_skill("英勇冲锋").is_ok());
    
    // Should be on cooldown now
    assert!(!hero.class_skills.is_skill_ready("英勇冲锋"));
    assert_eq!(hero.class_skills.get_cooldown("英勇冲锋"), 5);
    
    // Second use should fail (on cooldown)
    assert!(hero.try_use_skill("英勇冲锋").is_err());
}

#[test]
fn test_warrior_passive_regeneration() {
    let mut hero = Hero::new(Class::Warrior);
    
    // Reduce HP to below 50%
    hero.hp = hero.max_hp / 2 - 1;
    let initial_hp = hero.hp;
    
    // Advance turn - passive should trigger
    let _ = hero.on_turn();
    
    // HP should have increased due to regeneration passive
    assert!(hero.hp > initial_hp, "Warrior should regenerate when below 50% HP");
}

#[test]
fn test_hero_passive_perk_trigger_condition() {
    let mut hero = Hero::new(Class::Warrior);
    
    // Set HP to above 50% - passive should not trigger
    hero.hp = hero.max_hp - 1;
    let hp_before = hero.hp;
    let _ = hero.on_turn();
    
    // HP should not change (no healing when above 50%)
    assert_eq!(hero.hp, hp_before, "Warrior shouldn't regenerate when above 50% HP");
    
    // Set HP to below 50% - passive should trigger
    hero.hp = hero.max_hp / 2 - 5;
    let hp_before = hero.hp;
    let _ = hero.on_turn();
    
    // HP should increase
    assert!(hero.hp > hp_before, "Warrior should regenerate when below 50% HP");
}

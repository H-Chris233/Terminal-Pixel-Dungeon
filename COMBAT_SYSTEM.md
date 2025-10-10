# Terminal Pixel Dungeon - Combat System Documentation

## Overview

The combat system in Terminal Pixel Dungeon is designed to mimic the mechanics from Shattered Pixel Dungeon, implementing a turn-based system with strategic elements like ambushes, hit/miss calculation, and status effects.

## Core Components

### Combatant Trait
The `Combatant` trait defines the interface for any entity that can participate in combat:
- `hp()` and `max_hp()` - Health management
- `attack_power()` and `defense()` - Combat statistics
- `accuracy()` and `evasion()` - Hit/miss calculations
- `crit_bonus()` - Critical hit chance modifier
- `is_alive()` - Check if entity is alive
- `name()` - Entity identifier
- `attack_distance()` - Range of attacks
- `take_damage()` and `heal()` - Health modification methods
- `strength()`, `dexterity()`, `intelligence()` - Attributes

### Combat System
The core `Combat` struct handles combat resolution between two combatants. Key features:
- Hit/miss calculations using SPD-style formulas
- Critical hit mechanics
- Damage calculation with defense mitigation
- Ambush attack bonuses (2x damage)
- Combat logging for UI

## Combat Mechanics

### Turn-Based System
In keeping with SPD, the game uses a strict turn-based system where every player action costs one turn, and enemies get a turn in response. This ensures that every action is meaningful and strategic.

### Hit/Miss Calculation
The hit chance is calculated using this formula:
```
hit_chance = BASE_HIT_CHANCE + (accuracy - evasion) / 20
```
With a minimum and maximum cap to prevent guarantees.

### Ambush System
The vision system determines if an attacker can ambush a defender by:
1. Calculating the attacker's field of view (FOV)
2. Checking if the defender is visible to the attacker
3. If not visible, the attack is considered an ambush with 2x damage

### Damage Calculation
Damage is calculated with these factors:
- Base damage with random variation (80-120%)
- Critical hit multiplier (1.5x by default)
- Ambush bonus (2x by default)  
- Defense mitigation (with a cap at 80% reduction)

## Status Effects

The system supports various status effects:
- `Burning`: Causes damage each turn
- `Poison`: Causes damage each turn
- `Bleeding`: Causes damage each turn
- `Paralysis`: Prevents action
- `Invisibility`: Makes target harder to hit
- `Levitation`: Allows moving over traps
- `Slow`: Reduces action speed
- `Haste`: Increases action speed
- `MindVision`: See invisible enemies
- `AntiMagic`: Resist magical effects
- `Barkskin`: Temporary defense boost
- `Combo`: Bonus for consecutive hits
- `Fury`: Increases attack power
- `Ooze`: Ongoing damage
- `Frost`: Freezes target
- `Light`: Illuminates area
- `Darkness`: Reduces vision
- `Rooted`: Prevents movement

## Equipment Integration

Weapons and armor affect combat through:
- Damage bonuses from weapon quality
- Defense bonuses from armor
- Special properties from enchantments
- Attack distance from weapon type

## ECS Integration

The combat system integrates with the ECS through:
- The `CombatSystemBridge` that handles entity combat in the ECS world
- Vision system integration that works with dungeon terrain
- Status effects management using ECS component architecture

## Vision and Ambush

The vision system implements:
- Field of view calculation using Bresenham's line algorithm
- Line of sight checks to determine ambush opportunities
- Support for blocking terrain that affects visibility

## Usage Example

```rust
use combat::{Combat, Combatant};

// Example of engaging in combat
let mut attacker = /* ... */;
let mut defender = /* ... */;

let result = Combat::engage(
    &mut attacker,
    &mut defender,
    false  // is_ambush - normal attack
);

// Process combat results
for log in result.logs {
    println!("{}", log);
}
```

## Testing

The combat system includes comprehensive tests covering:
- Basic combat scenarios
- Ambush attacks
- Vision system functionality
- Status effect application
- ECS integration
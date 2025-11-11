# Combat Mechanics Documentation

## Overview

This document details the combat resolution system for the Battle Simulator. The system is designed to be historically accurate while maintaining computational efficiency for large-scale battles.

## Combat Resolution Pipeline

```
1. Detection Phase
   ├─→ Spatial Query (find nearby enemies)
   ├─→ Line of Sight Check
   └─→ Range Check

2. Target Selection Phase
   ├─→ Threat Assessment
   ├─→ Priority Calculation
   └─→ Target Assignment

3. Combat Resolution Phase
   ├─→ Calculate Hit Probability
   ├─→ Roll for Hits
   ├─→ Calculate Casualties
   └─→ Apply Damage

4. Morale Phase
   ├─→ Process Morale Events
   ├─→ Calculate Morale Changes
   └─→ Check for Rout/Rally
```

## Ranged Combat (Napoleonic Era)

### Hit Probability Formula

```rust
P(hit) = base_accuracy
       × distance_modifier
       × formation_modifier
       × morale_modifier
       × fatigue_modifier
       × terrain_modifier
       × weather_modifier
       × visibility_modifier
```

### Distance Modifier

Muskets follow a realistic accuracy falloff curve:

```rust
fn calculate_distance_modifier(distance: f32, effective_range: f32) -> f32 {
    let ratio = distance / effective_range;

    if ratio <= 0.5 {
        1.0  // Point blank: 100% effectiveness
    } else if ratio <= 1.0 {
        // Linear falloff from 0.5 to 1.0 range
        1.0 - (ratio - 0.5) * 0.6  // 100% → 70%
    } else if ratio <= 2.0 {
        // Exponential falloff beyond effective range
        0.7 * (-1.5 * (ratio - 1.0)).exp()  // 70% → 16%
    } else {
        // Minimal accuracy at long range
        0.05 * (1.0 / ratio)  // < 5%
    }
}
```

**Example Values (Musket, 50m effective range):**
| Distance | Modifier |
|----------|----------|
| 25m | 1.00 (100%) |
| 50m | 0.70 (70%) |
| 75m | 0.41 (41%) |
| 100m | 0.16 (16%) |
| 150m | 0.04 (4%) |
| 200m | 0.01 (1%) |

### Formation Modifier (Target)

Different formations present different target profiles:

```rust
pub enum FormationType {
    Line,      // 2-3 ranks deep, wide front
    Column,    // 8-12 ranks deep, narrow front
    Square,    // Hollow rectangle, anti-cavalry
    Skirmish,  // Dispersed individuals
}

fn get_target_formation_modifier(formation: FormationType) -> f32 {
    match formation {
        Line => 1.2,      // Large, linear target
        Column => 1.5,    // Dense, deep target (easier to hit)
        Square => 0.9,    // Defensive formation, disciplined
        Skirmish => 0.4,  // Dispersed, hard to hit
    }
}
```

### Morale Modifier (Shooter)

Unit morale affects shooting accuracy:

```rust
fn calculate_morale_modifier(morale: f32) -> f32 {
    // Square root scaling - morale has diminishing returns
    // 100 morale = 1.0x, 50 morale = 0.71x, 25 morale = 0.5x
    (morale / 100.0).sqrt()
}
```

### Fatigue Modifier

Physical exhaustion reduces combat effectiveness:

```rust
fn calculate_fatigue_modifier(fatigue: f32) -> f32 {
    // Linear penalty up to 30% at maximum fatigue
    1.0 - (fatigue / 100.0) * 0.3
}
```

**Fatigue accumulation:**
- Marching: +0.5 per minute
- Running: +2.0 per minute
- Combat: +1.0 per minute
- Resting: -1.5 per minute (up to baseline)

### Terrain Modifier

Terrain affects both attacker and defender:

```rust
pub enum TerrainType {
    Open,        // Fields, plains
    Woods,       // Light forest
    DenseWoods,  // Heavy forest
    Urban,       // Buildings
    Hill,        // Elevated position
}

fn get_terrain_modifier(
    shooter_terrain: TerrainType,
    target_terrain: TerrainType,
) -> f32 {
    let shooter_penalty = match shooter_terrain {
        Open => 1.0,
        Woods => 0.9,
        DenseWoods => 0.7,
        Urban => 0.95,
        Hill => 1.1,  // Elevation advantage
    };

    let target_cover = match target_terrain {
        Open => 1.0,
        Woods => 0.8,      // 20% cover
        DenseWoods => 0.6, // 40% cover
        Urban => 0.7,      // 30% cover
        Hill => 0.95,      // Slight cover from elevation
    };

    shooter_penalty * target_cover
}
```

### Volley Fire

Napoleonic combat is primarily volley fire, not individual shots:

```rust
fn resolve_volley(
    shooter: &Unit,
    target: &Unit,
    hit_probability: f32,
) -> u32 {
    let shots_fired = shooter.size as f32 * shooter.weapon.rate_of_fire;

    // Use Poisson distribution for realistic casualty variance
    // Lambda = expected hits = shots × probability
    let lambda = shots_fired * hit_probability;

    // Sample from Poisson distribution
    let hits = sample_poisson(lambda);

    hits
}
```

**Why Poisson?**
- Models rare events (hits) from many trials (shots)
- Provides realistic variance (sometimes lucky, sometimes unlucky)
- Matches historical casualty distributions

### Casualty Calculation

Not every hit causes a casualty:

```rust
fn calculate_casualties(
    hits: u32,
    target: &Unit,
    weapon: &Weapon,
) -> Casualties {
    let mut killed = 0;
    let mut wounded = 0;

    for _ in 0..hits {
        // Check if target's armor/cover stops the hit
        let penetration_chance = weapon.penetration
            * target.armor_modifier
            * target.cover_modifier;

        if random() < penetration_chance {
            // Hit penetrates - is it fatal?
            if random() < weapon.lethality {
                killed += 1;
            } else {
                wounded += 1;
            }
        }
    }

    Casualties {
        killed,
        wounded,
        effective_loss: killed + (wounded as f32 * 0.5) as u32,
    }
}
```

## Melee Combat (Napoleonic Era)

### Melee Resolution

Melee is resolved as opposed combat rolls:

```rust
fn resolve_melee(attacker: &Unit, defender: &Unit) -> MeleeResult {
    let attacker_power = calculate_combat_power(attacker, true);
    let defender_power = calculate_combat_power(defender, false);

    // Ratio determines outcome
    let power_ratio = attacker_power / defender_power;

    // Roll dice with modifiers
    let attacker_roll = roll_d100() * power_ratio;
    let defender_roll = roll_d100();

    let margin = attacker_roll - defender_roll;

    if margin > 20.0 {
        MeleeResult::DecisiveVictory(attacker)
    } else if margin > 0.0 {
        MeleeResult::MinorVictory(attacker)
    } else if margin > -20.0 {
        MeleeResult::MinorVictory(defender)
    } else {
        MeleeResult::DecisiveVictory(defender)
    }
}
```

### Combat Power Calculation

```rust
fn calculate_combat_power(unit: &Unit, is_attacker: bool) -> f32 {
    let mut power = unit.base_melee_skill;

    // Size matters in melee
    power *= (unit.size as f32).sqrt();

    // Morale is critical in melee
    power *= unit.morale / 100.0;

    // Fatigue heavily impacts melee
    power *= 1.0 - (unit.fatigue / 100.0) * 0.5;

    // Formation effectiveness
    power *= get_melee_formation_modifier(
        unit.formation,
        is_attacker,
    );

    // Charge bonus for attacker
    if is_attacker {
        power *= calculate_charge_bonus(unit.velocity);
    }

    // Terrain effects
    power *= get_melee_terrain_modifier(unit.position);

    power
}
```

### Formation Effectiveness in Melee

```rust
fn get_melee_formation_modifier(
    attacker_formation: FormationType,
    defender_formation: FormationType,
) -> f32 {
    match (attacker_formation, defender_formation) {
        // Column vs Line: Column wins (momentum)
        (Column, Line) => 1.3,
        (Line, Column) => 0.77,

        // Column vs Square: Square wins (bristling with bayonets)
        (Column, Square) => 0.6,
        (Square, Column) => 1.67,

        // Line vs Line: Even
        (Line, Line) => 1.0,

        // Skirmish is poor in melee
        (Skirmish, _) => 0.5,
        (_, Skirmish) => 2.0,

        // Cavalry modifiers (when implemented)
        // Cavalry vs Square: Disaster for cavalry
        // Cavalry vs Line: Devastating for cavalry
        // Cavalry vs Skirmish: Easy prey

        _ => 1.0,
    }
}
```

### Charge Bonus

Momentum from charging provides a significant advantage:

```rust
fn calculate_charge_bonus(velocity: f32) -> f32 {
    // Running speed ~2.5 m/s
    // Cavalry charge ~12 m/s

    let speed_ratio = velocity / 1.0; // Normalized to walking speed

    if speed_ratio > 2.0 {
        // Fast charge (running/cavalry)
        1.0 + (speed_ratio - 2.0) * 0.3
    } else if speed_ratio > 1.5 {
        // Medium pace (quick march)
        1.0 + (speed_ratio - 1.5) * 0.2
    } else {
        // Slow/stationary: no bonus
        1.0
    }
}
```

**Example values:**
| Movement | Speed | Bonus |
|----------|-------|-------|
| Stationary | 0 m/s | 1.0x (0%) |
| Walking | 1.2 m/s | 1.0x (0%) |
| Quick march | 1.8 m/s | 1.06x (+6%) |
| Running | 2.5 m/s | 1.15x (+15%) |
| Cavalry trot | 4.0 m/s | 1.6x (+60%) |
| Cavalry charge | 12.0 m/s | 4.0x (+300%) |

## Morale System

### Morale Events

Units experience morale changes from various events:

```rust
pub enum MoraleEvent {
    TakingCasualties {
        percentage_lost: f32,
    },
    FriendlyUnitRouted {
        distance: f32,
    },
    EnemyUnitRouted {
        distance: f32,
    },
    EnemyCharge {
        charging_unit_size: u32,
    },
    FlankAttack,
    RearAttack,
    CommanderKilled,
    CommanderPresent,
    VictoryInMelee,
    DefeatInMelee,
    RalliedByOfficer,
    UnderArtilleryFire,
}
```

### Morale Change Calculation

```rust
fn calculate_morale_change(
    unit: &Unit,
    event: &MoraleEvent,
) -> f32 {
    let base_change = match event {
        TakingCasualties { percentage_lost } => {
            -percentage_lost * 30.0
        },
        FriendlyUnitRouted { distance } => {
            let proximity = (500.0 - distance).max(0.0) / 500.0;
            -15.0 * proximity
        },
        EnemyUnitRouted { distance } => {
            let proximity = (500.0 - distance).max(0.0) / 500.0;
            15.0 * proximity
        },
        EnemyCharge { charging_unit_size } => {
            let size_factor = (*charging_unit_size as f32 / unit.size as f32);
            -10.0 * size_factor
        },
        FlankAttack => -20.0,
        RearAttack => -30.0,
        CommanderKilled => -25.0,
        CommanderPresent => 5.0,
        VictoryInMelee => 15.0,
        DefeatInMelee => -25.0,
        RalliedByOfficer => 10.0,
        UnderArtilleryFire => -5.0,
    };

    // Apply experience modifier
    let experience_modifier = match unit.experience_level {
        ExperienceLevel::Raw => 1.5,      // More affected
        ExperienceLevel::Trained => 1.0,   // Normal
        ExperienceLevel::Veteran => 0.7,   // More resilient
        ExperienceLevel::Elite => 0.5,     // Very resilient
    };

    // Apply unit quality modifier
    let quality_modifier = match unit.quality {
        UnitQuality::Poor => 1.3,
        UnitQuality::Average => 1.0,
        UnitQuality::Good => 0.8,
        UnitQuality::Elite => 0.6,
    };

    base_change * experience_modifier * quality_modifier
}
```

### Rout and Rally

```rust
fn check_morale_state(unit: &mut Unit) {
    if unit.morale < 20.0 {
        // Below rout threshold - chance to break
        let break_chance = 1.0 - (unit.morale / 20.0).powi(2);

        if random() < break_chance {
            unit.state = UnitState::Routing;
            tracing::warn!("{} has broken and is routing!", unit.name);
        }
    } else if unit.morale < 40.0 {
        // Wavering - penalties to combat effectiveness
        unit.state = UnitState::Wavering;
    } else if unit.state == UnitState::Wavering && unit.morale > 50.0 {
        // Recovered from wavering
        unit.state = UnitState::Steady;
    }

    // Routing units may rally if they get far from enemy
    if unit.state == UnitState::Routing {
        let distance_to_enemy = unit.distance_to_nearest_enemy();

        if distance_to_enemy > 300.0 && unit.morale > 30.0 {
            // Chance to rally
            let rally_chance = (unit.morale - 30.0) / 70.0;

            if random() < rally_chance * 0.1 { // 10% check per second
                unit.state = UnitState::Reforming;
                tracing::info!("{} is rallying!", unit.name);
            }
        }
    }
}
```

## Artillery

### Artillery Types (Napoleonic)

```rust
pub enum ArtilleryType {
    /// 6-12 pound field guns
    FieldGun {
        caliber_pounds: u32,
    },
    /// Heavy siege artillery
    SiegeGun {
        caliber_pounds: u32,
    },
    /// Light, mobile pieces
    HorseArtillery {
        caliber_pounds: u32,
    },
    /// Short-barreled mortars
    Howitzer {
        caliber_inches: f32,
    },
}
```

### Ammunition Types

```rust
pub enum ArtilleryAmmo {
    /// Solid iron ball - anti-structure, ricochet
    RoundShot {
        damage: f32,
        penetration: f32,
    },
    /// Anti-personnel at close range
    Canister {
        pellets: u32,
        spread: f32,
        effective_range: f32,
    },
    /// Like canister but smaller shot
    Grapeshot {
        pellets: u32,
        spread: f32,
        effective_range: f32,
    },
    /// Explosive shell (howitzers)
    Shell {
        explosive_radius: f32,
        fuse_time: f32,
    },
}
```

### Artillery Resolution

```rust
fn resolve_artillery_fire(
    gun: &ArtilleryUnit,
    target: &Unit,
    ammo_type: ArtilleryAmmo,
) -> CombatResult {
    match ammo_type {
        RoundShot { damage, penetration } => {
            // Single projectile - hits target unit or misses
            let hit_chance = calculate_artillery_accuracy(gun, target);

            if random() < hit_chance {
                // Cannonball plows through formation
                let casualties = calculate_roundshot_casualties(
                    target,
                    damage,
                    penetration,
                );

                // Morale impact from roundshot is significant
                let morale_loss = 5.0 + casualties as f32 * 2.0;

                CombatResult {
                    casualties,
                    morale_loss,
                }
            } else {
                CombatResult::default()
            }
        },
        Canister { pellets, spread, effective_range } => {
            // Shotgun blast - devastating at close range
            let distance = gun.position.distance(target.position);

            if distance > effective_range {
                return CombatResult::default();
            }

            // Effectiveness drops with distance
            let effectiveness = 1.0 - (distance / effective_range);

            // Each pellet is like a musket ball
            let hits = (pellets as f32 * effectiveness * 0.4) as u32;

            CombatResult {
                casualties: hits / 3, // ~33% lethality
                morale_loss: hits as f32 * 0.5,
            }
        },
        // ... other ammo types
    }
}
```

## Performance Considerations

### Spatial Optimization

Combat checks only happen between units in adjacent spatial cells:

```rust
fn process_combat_in_cell(
    cell: &SpatialCell,
    adjacent_cells: &[&SpatialCell],
) {
    // Only check combat within local area
    for unit in &cell.units {
        for adjacent_cell in adjacent_cells {
            for enemy in &adjacent_cell.enemies {
                if should_engage(unit, enemy) {
                    resolve_combat(unit, enemy);
                }
            }
        }
    }
}
```

### Level of Detail

Distant units use simplified combat:

```rust
fn get_combat_lod(distance_to_camera: f32) -> CombatDetail {
    match distance_to_camera {
        d if d < 500.0 => CombatDetail::Full,
        d if d < 2000.0 => CombatDetail::Simplified,
        _ => CombatDetail::Abstract,
    }
}
```

- **Full**: Individual casualty calculation, detailed morale
- **Simplified**: Batch calculations, approximate morale
- **Abstract**: Unit-level statistics only

### Parallelization

Combat is embarrassingly parallel:

```rust
// Process each spatial cell in parallel
spatial_grid.cells()
    .par_iter_mut()  // Rayon parallel iterator
    .for_each(|cell| {
        process_combat_in_cell(cell);
    });
```

## Historical Validation

Combat results should match historical casualty rates:

### Waterloo (1815)

| Force | Start | End | Casualties | Rate |
|-------|-------|-----|------------|------|
| French | 73,000 | 48,000 | 25,000 | 34% |
| Allied | 68,000 | 46,000 | 22,000 | 32% |
| Prussian | 50,000 | 43,000 | 7,000 | 14% |

### Borodino (1812)

| Force | Start | End | Casualties | Rate |
|-------|-------|-----|------------|------|
| French | 130,000 | 100,000 | 30,000 | 23% |
| Russian | 120,000 | 85,000 | 35,000 | 29% |

Simulator should produce similar casualty rates (±5%) for these scenarios.

## Future: WW1 Adaptations

The combat system will be extended for WW1:

- **Suppression system**: Machine guns pin units down
- **Artillery barrages**: Area effect, duration-based
- **Trench combat**: Positional warfare, attrition
- **Gas warfare**: Area denial, morale impact
- **Tanks**: Breakthrough units, infantry support

Core principles remain the same, but parameters change dramatically.

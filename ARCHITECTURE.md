# Battle Simulator Architecture

## Executive Summary

A high-performance, cross-platform battle simulator designed for historical accuracy, starting with the Napoleonic era and expanding to WW1. Built using Rust for maximum performance and safety, with an Entity Component System (ECS) architecture to handle thousands of units efficiently.

## Core Objectives

1. **Historical Accuracy**: Authentic unit capabilities, tactics, and combat mechanics
2. **Cross-Platform**: Windows, Linux, macOS support
3. **High Performance**: Handle 10,000+ units at 60+ FPS
4. **Extensibility**: Modular design to support multiple historical eras
5. **Scalability**: From small skirmishes to massive battles

## Technology Stack

### Primary Language: Rust
**Rationale:**
- Memory safety without garbage collection overhead
- Zero-cost abstractions for performance
- Excellent cross-platform support
- Strong ecosystem for game development
- Fearless concurrency for multi-threading

### Core Libraries
- **ECS Framework**: `bevy_ecs` or `specs` (Entity Component System)
- **Mathematics**: `glam` or `nalgebra` (SIMD-optimized linear algebra)
- **Serialization**: `serde` (data storage and configuration)
- **Parallel Processing**: `rayon` (work-stealing parallelism)
- **Spatial Indexing**: Custom quadtree/grid implementation

### Graphics & Rendering (Optional Layer)
- **Rendering**: `wgpu` (cross-platform WebGPU implementation)
- **UI**: `egui` (immediate mode GUI)
- **Asset Loading**: `bevy_asset` or custom loader

### Build & Deployment
- **Build System**: Cargo with workspace configuration
- **CI/CD**: GitHub Actions for multi-platform builds
- **Packaging**: Platform-specific installers (MSI, DEB, DMG)

## System Architecture

### 1. Layered Architecture

```
┌─────────────────────────────────────────┐
│         Presentation Layer              │
│  (Graphics, UI, Input Handling)         │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│       Application Layer                 │
│  (Game Loop, State Management)          │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│       Simulation Engine                 │
│  (Combat, Movement, AI, Orders)         │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│          Core Systems                   │
│  (ECS, Physics, Spatial Index)          │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│          Data Layer                     │
│  (Units, Weapons, Scenarios, Maps)      │
└─────────────────────────────────────────┘
```

### 2. Entity Component System (ECS) Design

**Why ECS?**
- Data-oriented design for cache efficiency
- Easy parallelization of systems
- Flexible composition of unit behaviors
- Excellent performance with large entity counts

**Core Components:**

```rust
// Position & Movement
Position { x, y, z, facing }
Velocity { dx, dy, speed }
Formation { type, position_in_formation, spacing }

// Combat
Health { current, maximum, armor }
Weapon {
    type, range, accuracy, reload_time,
    last_fired, ammunition
}
CombatStats {
    melee_skill, ranged_skill, defense
}

// Unit Organization
UnitIdentity {
    id, name, era, nation, unit_type
}
Squad {
    size, casualties, cohesion
}
Command {
    commander_id, subordinates, rank
}

// Psychology & Morale
Morale {
    current, base, modifiers
}
Fatigue {
    current, max, recovery_rate
}
Experience {
    level, battles, training
}

// AI & Orders
AIState {
    current_order, target, state_machine
}
PathfindingAgent {
    current_path, destination
}
```

**Core Systems:**

```
MovementSystem      → Updates positions based on orders
CombatSystem        → Resolves combat, calculates casualties
MoraleSystem        → Updates morale, handles routing
FormationSystem     → Maintains unit formations
AISystem            → Processes unit AI and orders
SpatialIndexSystem  → Updates spatial partitioning
LineOfSightSystem   → Calculates visibility
ProjectileSystem    → Handles bullet/cannonball trajectories
```

### 3. Performance Optimization Strategy

#### Spatial Partitioning
```
- Uniform Grid (primary): 100m x 100m cells
- Quadtree (secondary): For hierarchical queries
- Only check combat in adjacent cells
- O(1) insertion/removal, O(k) neighbor queries
```

#### Multi-threading Strategy
```
Frame N:
┌────────────────────────────────────────┐
│ Physics Update (parallel per cell)     │
│ Combat Resolution (parallel per cell)  │
│ AI Decisions (parallel per unit type)  │
│ Spatial Index Update (single-threaded) │
│ Morale Updates (parallel)              │
└────────────────────────────────────────┘

- Use rayon for data parallelism
- Thread pool sized to CPU cores
- Lock-free reads where possible
- Batch writes to avoid contention
```

#### Memory Layout Optimization
```
- Structure of Arrays (SoA) layout in ECS
- Cache-aligned components
- Component archetypes for iteration efficiency
- Memory pools for frequent allocations
```

#### Level of Detail (LOD)
```
Distance-based simulation fidelity:
- 0-500m: Individual soldier simulation
- 500-2000m: Squad-level abstraction
- 2000m+: Unit-level abstraction
- Automatic degradation under load
```

### 4. Combat Simulation Model

#### Napoleonic Era Combat Mechanics

**Formation Types:**
- Line (2-3 ranks deep): Maximum firepower
- Column: Maximum shock, mobility
- Square: Anti-cavalry defense
- Skirmish: Dispersed, reduced casualties

**Combat Resolution (per tick):**

```rust
fn resolve_ranged_combat(
    shooter: &Unit,
    target: &Unit,
    distance: f32,
    visibility: f32
) -> CombatResult {
    let base_accuracy = shooter.weapon.accuracy;

    // Distance falloff (smooth musket curve)
    let distance_modifier = calculate_distance_modifier(
        distance,
        shooter.weapon.effective_range
    );

    // Formation modifier
    let formation_modifier = match target.formation {
        Formation::Line => 1.2,    // Large target
        Formation::Column => 1.5,  // Dense target
        Formation::Square => 0.9,  // Defensive
        Formation::Skirmish => 0.4 // Dispersed
    };

    // Morale & fatigue effects
    let morale_modifier = (shooter.morale / 100.0).powf(0.5);
    let fatigue_modifier = 1.0 - (shooter.fatigue / 100.0) * 0.3;

    // Environmental factors
    let terrain_modifier = get_terrain_modifier(shooter.position);
    let weather_modifier = get_weather_modifier();

    let hit_chance = base_accuracy
        * distance_modifier
        * formation_modifier
        * morale_modifier
        * fatigue_modifier
        * terrain_modifier
        * weather_modifier
        * visibility;

    // Roll for hits (Poisson distribution for volley fire)
    let shots_fired = shooter.squad.size * shooter.weapon.rate_of_fire;
    let hits = sample_poisson(shots_fired * hit_chance);

    // Calculate casualties (considering armor/cover)
    calculate_casualties(hits, target)
}
```

**Melee Combat:**
```rust
fn resolve_melee_combat(
    attacker: &Unit,
    defender: &Unit
) -> CombatResult {
    // Charge momentum
    let charge_bonus = calculate_charge_bonus(attacker.velocity);

    // Relative strength
    let strength_ratio = attacker.squad.size / defender.squad.size;

    // Formation effectiveness in melee
    let formation_effectiveness = match (attacker.formation, defender.formation) {
        (Column, Line) => 1.3,      // Column smashes line
        (Column, Square) => 0.6,    // Column vs square fails
        (Cavalry, Square) => 0.3,   // Cavalry vs square very bad
        (Cavalry, Line) => 2.0,     // Cavalry vs line devastating
        // ... more combinations
    };

    let combat_power =
        attacker.combat_stats.melee_skill
        * attacker.morale
        * strength_ratio
        * charge_bonus
        * formation_effectiveness;

    // Opposed roll
    resolve_opposed_combat(combat_power, defender)
}
```

**Morale System:**
```rust
struct MoraleEvent {
    event_type: MoraleEventType,
    severity: f32
}

enum MoraleEventType {
    TakingCasualties(f32),      // % of unit lost
    FriendlyUnitRouted,         // Nearby unit breaks
    EnemyCharge,                 // Being charged
    FlankAttack,                 // Attacked from side/rear
    CommanderKilled,             // Leader casualty
    Victory,                     // Enemy unit breaks
    Rallied,                     // Officer rally attempt
}

fn update_morale(unit: &mut Unit, events: &[MoraleEvent]) {
    let mut morale_delta = 0.0;

    for event in events {
        morale_delta += match event.event_type {
            TakingCasualties(pct) => -pct * 30.0,
            FriendlyUnitRouted => -15.0,
            EnemyCharge => -10.0 * event.severity,
            FlankAttack => -20.0,
            CommanderKilled => -25.0,
            Victory => +15.0,
            Rallied => +10.0,
        };
    }

    // Apply morale modifiers
    morale_delta *= unit.experience.modifier();
    morale_delta *= get_unit_quality_modifier(unit);

    unit.morale = (unit.morale + morale_delta).clamp(0.0, 100.0);

    // Check for rout
    if unit.morale < 20.0 && rng.gen::<f32>() > unit.morale / 100.0 {
        unit.set_state(UnitState::Routing);
    }
}
```

### 5. AI System

**Hierarchical AI Architecture:**

```
Strategic AI (Battle-level)
    ↓
Operational AI (Brigade/Division-level)
    ↓
Tactical AI (Battalion/Regiment-level)
    ↓
Squad AI (Individual unit behavior)
```

**Decision Making:**
- **Utility-based AI**: Score multiple actions
- **Behavior Trees**: For tactical execution
- **Goal-oriented Action Planning**: For operational decisions

**Example Tactical AI:**
```rust
fn evaluate_tactical_options(unit: &Unit, context: &BattleContext) -> Action {
    let mut options = vec![
        (Action::Hold, evaluate_hold_position(unit, context)),
        (Action::Advance, evaluate_advance(unit, context)),
        (Action::Retreat, evaluate_retreat(unit, context)),
        (Action::ChangeFormation(f), evaluate_formation_change(unit, f, context)),
        (Action::Fire, evaluate_fire(unit, context)),
    ];

    // Find highest utility action
    options.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    options[0].0
}

fn evaluate_advance(unit: &Unit, context: &BattleContext) -> f32 {
    let mut score = 0.0;

    // Strong motivation if enemy is weak/routing
    let enemy_morale = context.get_nearest_enemy_morale(unit);
    score += (100.0 - enemy_morale) * 0.5;

    // Penalty if low morale/fatigue
    score -= (100.0 - unit.morale) * 0.3;
    score -= unit.fatigue * 0.2;

    // Bonus if we have numerical superiority
    let local_superiority = context.calculate_local_force_ratio(unit);
    score += (local_superiority - 1.0) * 50.0;

    // Consider orders from commander
    if unit.orders.contains(Order::Attack) {
        score += 30.0;
    }

    score
}
```

### 6. Extensibility Framework for Multiple Eras

**Era-Based Plugin System:**

```rust
trait Era {
    fn name(&self) -> &str;
    fn time_period(&self) -> (i32, i32); // Start, end years

    fn get_unit_types(&self) -> Vec<UnitType>;
    fn get_combat_resolver(&self) -> Box<dyn CombatResolver>;
    fn get_weapon_database(&self) -> WeaponDatabase;
    fn get_formation_rules(&self) -> FormationRules;

    fn create_systems(&self, world: &mut World) -> Vec<Box<dyn System>>;
}

struct NapoleonicEra;
impl Era for NapoleonicEra {
    fn name(&self) -> &str { "Napoleonic Wars" }
    fn time_period(&self) -> (i32, i32) { (1803, 1815) }

    fn get_combat_resolver(&self) -> Box<dyn CombatResolver> {
        Box::new(NapoleonicCombatResolver::new())
    }
    // ... implementations
}

struct WW1Era;
impl Era for WW1Era {
    fn name(&self) -> &str { "World War 1" }
    fn time_period(&self) -> (i32, i32) { (1914, 1918) }

    fn get_combat_resolver(&self) -> Box<dyn CombatResolver> {
        Box::new(WW1CombatResolver::new())
    }
    // ... different combat mechanics for trenches, machine guns, etc.
}
```

**Era-Specific Differences:**

| Aspect | Napoleonic | WW1 |
|--------|-----------|-----|
| Formation | Line, Column, Square | Trench, Skirmish, Wave |
| Weapons | Musket, Cannon | Rifle, Machine Gun, Artillery |
| Combat Range | 50-200m | 300-1000m |
| Morale Model | Unit-based | Suppression-based |
| Movement | Formation-based | Cover-to-cover |
| Key Mechanics | Cavalry charges, Bayonet | Artillery barrages, Gas |

### 7. Data Schema

**Unit Definition (TOML/JSON):**
```toml
[unit.french_line_infantry]
name = "French Line Infantry"
era = "Napoleonic"
nation = "France"
type = "Infantry"
period = [1803, 1815]

[unit.french_line_infantry.stats]
base_size = 800
morale = 75
experience = 60
melee_skill = 70
ranged_skill = 65
speed = 1.2  # m/s
stamina = 80

[unit.french_line_infantry.weapon]
type = "Charleville_Musket"
range = 100  # meters
effective_range = 50
accuracy = 0.4  # 40% at effective range
reload_time = 15  # seconds
ammunition = 60

[unit.french_line_infantry.formations]
available = ["Line", "Column", "Square", "Skirmish"]
default = "Column"

[unit.french_line_infantry.cost]
recruitment = 500
upkeep = 50
```

**Weapon Database:**
```toml
[weapon.charleville_musket]
name = "Charleville Musket Model 1777"
type = "smoothbore_musket"
caliber = 17.5  # mm
muzzle_velocity = 400  # m/s
rate_of_fire = 3  # rounds per minute
effective_range = 50
maximum_range = 200
accuracy_curve = "musket_standard"  # Reference to accuracy model
penetration = 0.2  # Low armor penetration

[weapon.12_pounder_cannon]
name = "12-Pounder Cannon"
type = "smoothbore_artillery"
caliber = 121  # mm
crew_size = 8
rate_of_fire = 1.5  # rounds per minute
effective_range = 800
maximum_range = 1800
ammunition_types = ["round_shot", "canister", "grape"]

[weapon.12_pounder_cannon.round_shot]
damage = 100
penetration = 0.8
area_of_effect = 2  # meters

[weapon.12_pounder_cannon.canister]
damage = 50
penetration = 0.1
area_of_effect = 15
effective_range = 300
```

### 8. Project Structure

```
battle-simulator/
├── Cargo.toml                 # Workspace configuration
├── ARCHITECTURE.md            # This document
├── README.md
│
├── crates/
│   ├── core/                  # Core ECS and engine
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── ecs/           # ECS implementation
│   │   │   ├── spatial/       # Spatial indexing
│   │   │   └── math/          # Math utilities
│   │   └── Cargo.toml
│   │
│   ├── simulation/            # Battle simulation logic
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── combat/        # Combat resolution
│   │   │   ├── movement/      # Movement & pathfinding
│   │   │   ├── morale/        # Morale system
│   │   │   ├── ai/            # AI systems
│   │   │   └── orders/        # Command & control
│   │   └── Cargo.toml
│   │
│   ├── era_napoleonic/        # Napoleonic era plugin
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── units/         # Unit definitions
│   │   │   ├── combat.rs      # Era-specific combat
│   │   │   └── formations.rs  # Formation rules
│   │   ├── data/              # Data files
│   │   │   ├── units/
│   │   │   ├── weapons/
│   │   │   └── scenarios/
│   │   └── Cargo.toml
│   │
│   ├── era_ww1/               # WW1 era plugin (future)
│   │   └── ...
│   │
│   ├── renderer/              # Graphics rendering (optional)
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── gpu/           # GPU rendering
│   │   │   ├── ui/            # UI systems
│   │   │   └── assets/        # Asset loading
│   │   └── Cargo.toml
│   │
│   └── cli/                   # Command-line interface
│       ├── src/
│       │   └── main.rs
│       └── Cargo.toml
│
├── data/                      # Shared data files
│   ├── maps/
│   ├── scenarios/
│   └── config/
│
├── docs/                      # Documentation
│   ├── combat_mechanics.md
│   ├── ai_design.md
│   └── era_guide.md
│
├── tests/                     # Integration tests
│   ├── combat_tests.rs
│   ├── performance_tests.rs
│   └── accuracy_tests.rs
│
└── benches/                   # Performance benchmarks
    ├── spatial_index.rs
    ├── combat_resolution.rs
    └── large_battle.rs
```

### 9. Performance Targets

**Baseline Hardware:** Mid-range desktop (Ryzen 5 / Intel i5, 16GB RAM)

| Scenario | Units | Target FPS | Memory |
|----------|-------|------------|--------|
| Small Skirmish | 500 | 120+ | <500MB |
| Medium Battle | 2,000 | 90+ | <1GB |
| Large Battle | 5,000 | 60+ | <2GB |
| Epic Battle | 10,000 | 30+ | <3GB |
| Massive Campaign | 50,000 | 15+ | <8GB |

**Optimization Checkpoints:**
- Profile every 1000 units added
- Maintain <16ms frame time for 60 FPS
- CPU: <70% utilization on 4 cores
- Memory: Linear scaling with unit count

### 10. Development Roadmap

**Phase 1: Core Foundation (Months 1-2)**
- ECS implementation
- Basic rendering/visualization
- Spatial partitioning
- Movement system
- Basic combat mechanics

**Phase 2: Napoleonic Combat (Months 3-4)**
- Formation system
- Ranged combat (muskets, cannons)
- Melee combat
- Morale system
- Unit data for major nations

**Phase 3: AI & Command (Months 5-6)**
- Basic tactical AI
- Order system
- Command hierarchy
- Pathfinding
- Simple strategic AI

**Phase 4: Polish & Scenarios (Month 7)**
- Historical scenarios (Waterloo, Austerlitz, etc.)
- Performance optimization
- UI improvements
- Testing & balancing

**Phase 5: WW1 Foundation (Months 8-9)**
- Era plugin system finalization
- WW1 unit database
- Trench warfare mechanics
- Artillery barrage system
- Machine gun suppression

**Phase 6: WW1 Completion (Months 10-12)**
- WW1 scenarios (Somme, Verdun, etc.)
- Advanced AI for WW1 tactics
- Gas warfare
- Tank mechanics
- Final optimization & release

### 11. Testing Strategy

**Unit Tests:**
- Combat calculations
- Morale calculations
- Formation logic
- Spatial indexing

**Integration Tests:**
- Complete battle scenarios
- AI decision making
- Performance benchmarks
- Cross-platform compatibility

**Historical Accuracy Tests:**
- Compare casualty rates to historical data
- Formation effectiveness validation
- Weapon performance validation
- Battle outcome reproduction

**Performance Tests:**
```rust
#[bench]
fn bench_large_battle(b: &mut Bencher) {
    let world = create_battle(10_000_units);
    b.iter(|| {
        simulate_tick(&mut world);
    });
    assert!(b.elapsed() < Duration::from_millis(16)); // <16ms
}
```

### 12. Historical Accuracy Sources

**Primary References:**
- Nafziger Order of Battle Collection
- "The Anatomy of Victory" - Rory Muir
- "Battle Tactics of Napoleon and His Enemies" - Brent Nosworthy
- "On Artillery" - Antoine-Henri Jomini
- Military drill manuals (1791-1815)
- WW1: "Infantry Attacks" - Erwin Rommel
- WW1: British Tactical Manuals (1914-1918)

**Validation:**
- Unit rosters from historical battles
- Casualty rate comparisons
- Timeline of battle events
- Formation deployment patterns

## Conclusion

This architecture provides:
- **Performance**: ECS + spatial partitioning + multi-threading = 10,000+ units
- **Accuracy**: Historical data-driven combat models
- **Extensibility**: Era plugin system for Napoleonic → WW1 → beyond
- **Maintainability**: Clean separation of concerns, modular design
- **Cross-platform**: Rust + cross-platform libraries

The system is designed to scale from small skirmishes to massive battles while maintaining historical fidelity and exceptional performance.

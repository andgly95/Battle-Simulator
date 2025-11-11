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

### 8. Rendering Architecture

The rendering system is designed with a clean abstraction layer to support multiple backends, scaling from simple 2D pixel visualization to full 3D animated scenes with minimal changes to the simulation core.

#### 8.1 Design Principles

**Separation of Concerns:**
- Simulation runs independently of rendering
- Renderer reads from ECS components without modifying them
- Frame rate can differ from simulation tick rate
- Renderer can be swapped without changing simulation logic

**Performance First:**
- Instanced rendering for massive unit counts (one draw call per unit type)
- Spatial culling - only render visible units
- Level of Detail (LOD) system:
  - Far: Single pixel per unit
  - Medium: Sprite/simple geometry
  - Close: Detailed models with animations
- Render on separate thread from simulation

**Scalability Path:**
```
Phase 1: 2D Pixels    →  Phase 2: 2D Sprites  →  Phase 3: 3D Models
(pixels crate)           (wgpu + batching)        (wgpu + instancing)
```

#### 8.2 Rendering Abstraction Layer

**Core Trait:**
```rust
/// Abstract renderer that can be implemented by different backends
pub trait Renderer {
    /// Initialize the rendering backend
    fn initialize(&mut self, width: u32, height: u32) -> Result<()>;

    /// Begin a new frame
    fn begin_frame(&mut self);

    /// Render all units from the simulation
    fn render_units(&mut self, units: &Query<(&Position, &Team, &Squad, &AIState)>);

    /// Render terrain/map
    fn render_terrain(&mut self, terrain: &TerrainData);

    /// Render UI overlay (stats, minimap)
    fn render_ui(&mut self, stats: &BattleStatistics);

    /// Complete frame and present to screen
    fn end_frame(&mut self) -> Result<()>;

    /// Handle window resize
    fn resize(&mut self, width: u32, height: u32);

    /// Get camera controller
    fn camera_mut(&mut self) -> &mut Camera;
}

/// Camera system for view control
pub struct Camera {
    pub position: Vec2,      // World position camera is looking at
    pub zoom: f32,           // Zoom level (1.0 = 1 pixel = 1 meter)
    pub rotation: f32,       // Camera rotation in radians
    pub viewport: (u32, u32), // Screen size in pixels
}

impl Camera {
    /// Convert world coordinates to screen coordinates
    pub fn world_to_screen(&self, world_pos: Vec2) -> Vec2;

    /// Convert screen coordinates to world coordinates
    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2;

    /// Pan camera by screen delta
    pub fn pan(&mut self, delta: Vec2);

    /// Zoom in/out (mouse wheel)
    pub fn zoom_at(&mut self, screen_pos: Vec2, zoom_delta: f32);

    /// Get visible world bounds for culling
    pub fn visible_bounds(&self) -> Rect;
}
```

#### 8.3 Backend Implementations

**Phase 1: Pixel Renderer (Current)**
```rust
/// Simple 2D pixel-based renderer using the `pixels` crate
pub struct PixelRenderer {
    pixels: Pixels,           // Pixel buffer
    camera: Camera,
    width: u32,
    height: u32,
}

impl Renderer for PixelRenderer {
    fn render_units(&mut self, units: &Query<...>) {
        let frame = self.pixels.get_frame_mut();

        // Clear to background
        frame.fill(0);

        // Get visible bounds for culling
        let visible = self.camera.visible_bounds();

        for (pos, team, squad, ai_state) in units.iter() {
            // Skip units outside camera view
            if !visible.contains(pos.x, pos.y) {
                continue;
            }

            // Convert world position to screen
            let screen_pos = self.camera.world_to_screen(Vec2::new(pos.x, pos.y));

            // Determine color based on team and state
            let color = match (team.side, ai_state.state) {
                (Side::Allied, BehaviorState::Routing) => BLUE_ROUTING,
                (Side::Allied, _) => BLUE_ACTIVE,
                (Side::Enemy, BehaviorState::Routing) => RED_ROUTING,
                (Side::Enemy, _) => RED_ACTIVE,
            };

            // Draw unit (simple rectangle for now)
            self.draw_rect(frame, screen_pos, 4, 4, color);
        }
    }
}
```

**Phase 2: Sprite Renderer (Future)**
```rust
/// 2D sprite-based renderer using wgpu
pub struct SpriteRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    sprite_batch: SpriteBatch,  // Batches sprites for instanced rendering
    camera: Camera,
    textures: HashMap<String, Texture>,
}

impl Renderer for SpriteRenderer {
    fn render_units(&mut self, units: &Query<...>) {
        self.sprite_batch.clear();

        for (pos, team, formation, facing) in units.iter() {
            // Choose sprite based on unit type and formation
            let sprite_id = match formation.formation_type {
                FormationType::Line => "infantry_line",
                FormationType::Column => "infantry_column",
                // ...
            };

            // Add to batch (all units drawn in one call per sprite type)
            self.sprite_batch.add(
                sprite_id,
                pos.x, pos.y,
                facing.angle,
                team_color(team),
            );
        }

        // Submit batched draw calls
        self.sprite_batch.draw(&self.device, &self.queue);
    }
}
```

**Phase 3: 3D Renderer (Future)**
```rust
/// Full 3D renderer with animated models
pub struct Renderer3D {
    device: wgpu::Device,
    queue: wgpu::Queue,
    models: HashMap<String, Model3D>,
    instance_buffer: InstanceBuffer,  // GPU buffer for instanced rendering
    camera: Camera3D,  // Perspective camera
}

impl Renderer for Renderer3D {
    fn render_units(&mut self, units: &Query<...>) {
        // Collect all instances per model type
        let mut instances_by_model: HashMap<&str, Vec<Instance>> = HashMap::new();

        for (pos, unit_id, facing, animation_state) in units.iter() {
            let model_id = &unit_id.unit_type;

            instances_by_model
                .entry(model_id)
                .or_default()
                .push(Instance {
                    position: [pos.x, pos.y, pos.z],
                    rotation: facing.angle,
                    animation_frame: animation_state.frame,
                });
        }

        // Draw all instances of each model type in one call
        for (model_id, instances) in instances_by_model {
            self.instance_buffer.update(&instances);
            self.models[model_id].draw_instanced(&self.device, instances.len());
        }
    }
}
```

#### 8.4 Performance Optimizations

**Instanced Rendering:**
- Single draw call for all units of the same type
- GPU-side transformations
- Target: 10,000+ units at 60 FPS

**Spatial Culling:**
```rust
// Only render units in camera view
let visible_bounds = camera.visible_bounds();
let visible_entities = spatial_index.query_rect(visible_bounds);

// Render only visible entities
for entity in visible_entities {
    // ...
}
```

**Level of Detail (LOD):**
```rust
fn determine_lod(distance_to_camera: f32) -> LOD {
    match distance_to_camera {
        0.0..=100.0 => LOD::High,      // Full detail, animations
        100.0..=500.0 => LOD::Medium,  // Simple sprites
        500.0..=2000.0 => LOD::Low,    // Single pixel/dot
        _ => LOD::None,                // Don't render (culled)
    }
}
```

**Double Buffering:**
- Simulation writes to one buffer
- Renderer reads from previous buffer
- No locks during frame rendering

**Dirty Tracking:**
```rust
// Only update static terrain once
if terrain.is_dirty() {
    renderer.update_terrain(terrain);
    terrain.mark_clean();
}
```

#### 8.5 Integration Pattern

**Main Loop:**
```rust
fn main() {
    let mut world = World::new();
    let mut renderer = PixelRenderer::new(1920, 1080);
    let mut schedule = create_simulation_schedule();

    let mut last_sim_tick = Instant::now();
    let sim_rate = Duration::from_millis(33);  // 30 Hz simulation

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::RedrawRequested(_) => {
                // Run simulation ticks if needed
                while last_sim_tick.elapsed() >= sim_rate {
                    schedule.run(&mut world);
                    last_sim_tick += sim_rate;
                }

                // Render current state
                renderer.begin_frame();

                let mut unit_query = world.query::<(&Position, &Team, &Squad, &AIState)>();
                renderer.render_units(&unit_query);

                let stats = world.resource::<BattleStatistics>();
                renderer.render_ui(stats);

                renderer.end_frame().unwrap();
            }

            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                renderer.resize(size.width, size.height);
            }

            // Handle input...
        }
    });
}
```

#### 8.6 Rendering Features

**Phase 1 (2D Pixels):**
- ✅ Colored rectangles for units (team colors)
- ✅ Camera pan/zoom
- ✅ Unit state visualization (routing = darker color)
- ✅ Simple terrain (grid lines)
- ✅ Basic UI overlay (stats, FPS counter)

**Phase 2 (2D Sprites):**
- □ Sprite-based unit rendering
- □ Formation visualization (units arranged in proper formations)
- □ Directional sprites (facing direction)
- □ Simple animations (march, fire, melee)
- □ Terrain textures
- □ Minimap
- □ Projectile trails

**Phase 3 (3D Models):**
- □ Full 3D animated models
- □ Detailed terrain with elevation
- □ Particle effects (smoke, explosions)
- □ Dynamic lighting
- □ Weather effects
- □ Cinematic camera
- □ Unit damage visualization

#### 8.7 Dependencies

**Phase 1:**
```toml
[dependencies]
pixels = "0.13"           # 2D pixel buffer rendering
winit = "0.29"            # Window creation and event handling
glam = "0.24"             # Math library for camera
```

**Phase 2:**
```toml
[dependencies]
wgpu = "0.18"             # Modern GPU API (WebGPU)
wgpu-types = "0.18"
winit = "0.29"
glam = "0.24"
image = "0.24"            # Image loading for textures
```

**Phase 3:**
```toml
[dependencies]
wgpu = "0.18"
glam = "0.24"
gltf = "1.4"              # 3D model loading
cgmath = "0.18"           # Additional math for 3D
```

### 9. Project Structure

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

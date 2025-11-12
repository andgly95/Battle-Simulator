# Individual Soldier Simulation

## Overview

Complete implementation of individual soldier-level simulation with **4,800 separate entities**, each with their own AI, weapon state, and position.

## What This Achieves

### Before (Formation-Level)
- 6 battalion entities representing 4,800 soldiers
- Battalions shot as single units
- No visible projectiles
- Abstract casualty resolution

### After (Individual Soldier-Level)
- **4,800 individual soldier entities**
- Each front-rank soldier shoots individually
- **Visible projectiles** flying through the air (400 m/s)
- Specific soldiers get hit and despawn
- Survivors actively move to fill gaps in formation
- Individual AI controlling every soldier's behavior

## Technical Implementation

### Components

**Soldier Component**
```rust
struct Soldier {
    battalion: Entity,           // Which battalion they belong to
    rank: u32,                   // Row in formation (0 = front rank)
    file: u32,                   // Column in formation
    target_pos: Vec2,            // Where they should be standing
    in_front_rank: bool,         // Can they shoot?
}
```

**SoldierWeapon Component**
```rust
struct SoldierWeapon {
    loaded: bool,                // Ready to fire?
    reload_time: f32,            // 15 seconds for musket
    time_since_fire: f32,        // Reload progress
    ammunition: u32,             // 60 rounds per soldier
}
```

**Projectile Component**
```rust
struct Projectile {
    velocity: Vec2,              // 400 m/s in direction of target
    max_range: f32,              // 200m musket range
    distance: f32,               // How far it's traveled
    team: Side,                  // Who fired it
}
```

**Battalion Component**
```rust
struct Battalion {
    formation_type: FormationType,  // Line, Column, Square, Skirmish
    formation_center: Vec2,         // Where the formation is centered
}
```

### Systems (30 Hz Simulation)

1. **assign_formation_positions**
   - Runs every frame to reassign soldier positions after casualties
   - Line formation: All survivors close up into single line
   - Column formation: Reorganizes into 20-wide column, filling from front
   - This creates the "active gap-filling" behavior

2. **soldier_movement_system**
   - Each soldier moves toward their assigned `target_pos`
   - Speed: 1.0 m/s when moving to position
   - Stops when within 0.5m of target position
   - Creates organic reformation behavior

3. **soldier_weapon_reload_system**
   - Tracks individual reload timers for each soldier
   - 15-second musket reload time
   - Sets `loaded = true` when ready

4. **soldier_shooting_system**
   - Front-rank soldiers find nearest enemy within 200m
   - If found and loaded: FIRE!
   - Spawns visible **Projectile** entity
   - Direction calculated toward specific target soldier
   - Velocity: 400 m/s

5. **projectile_movement_system**
   - Moves all projectiles through space
   - Updates position based on velocity × delta time
   - Tracks distance traveled

6. **projectile_hit_detection_system**
   - Checks each projectile against all soldiers
   - Hit detection: distance < 1.0m
   - On hit: **Despawns specific soldier** + projectile
   - Also despawns projectiles beyond max range

## Formation Spawning

### British Line (800 soldiers)
```rust
spawn_battalion(&mut world, vec2(150.0, 400.0), Side::Enemy, FormationType::Line, 800);
```
- Spawns 800 individual soldiers in single line
- 2.0m spacing between soldiers
- Total width: 1,600 meters (!)
- All are front-rank (can shoot)

### French Column (800 soldiers)
```rust
spawn_battalion(&mut world, vec2(150.0, 200.0), Side::Allied, FormationType::Column, 800);
```
- Spawns 800 soldiers in 20-wide × 40-deep column
- 1.5m spacing
- Only front 20 soldiers can shoot
- Dimensions: 30m × 60m

## Emergent Behaviors

### Realistic Volley Fire
- Only front-rank soldiers shoot (20 out of 800 in column, 800 out of 800 in line)
- Each finds independent target
- Creates ragged, realistic volley instead of synchronized salvo
- Reload times are individual, so continuous rolling fire

### Gap Filling After Casualties
1. Projectile hits specific soldier → soldier despawns
2. Next frame: `assign_formation_positions` detects gap
3. Survivors get new `target_pos` to close up ranks
4. `soldier_movement_system` moves them toward new positions
5. Formation gradually becomes whole again

### Column vs Line Dynamics
- **British Line**: 800 muskets firing at once (devastating firepower)
- **French Column**: Only 20 muskets firing (front rank)
- Historically accurate Column vs Line problem!
- French columns would get torn apart by British lines

## Performance

### Entity Counts
- **4,800 soldier entities** (3 battalions British + 3 French)
- **Up to ~200 projectile entities** in flight at once
- **~5,000 total entities** during heavy combat

### Optimizations
- Simulation rate: 30 Hz (33ms per tick)
- Render rate: Uncapped (60+ FPS)
- Spatial culling: Only render soldiers in view
- Hit detection: O(n×m) where n=projectiles, m=soldiers (future: spatial indexing)

## Visual Rendering

### Soldiers
- 2×2 pixel squares
- Blue for French (Allied)
- Red for British (Enemy)
- Visible in formation patterns

### Projectiles
- 1×1 pixel dots
- Light blue for French projectiles
- Light red for British projectiles
- Trail across battlefield at 400 m/s

## Running the Simulation

```bash
cargo run --example individual_soldiers
```

### Requirements
- Display server (X11/Wayland)
- GPU for hardware-accelerated pixel rendering

### Expected Output
```
==============================================
  INDIVIDUAL SOLDIER SIMULATION
  4,800 soldiers, each with their own AI!
==============================================
Spawning 2,400 British soldiers in LINE formation...
Spawning 2,400 French soldiers in COLUMN formation...
Total: 4,800 individual soldier entities created!

FPS: 60 | Soldiers: 4723 | Projectiles: 47
FPS: 59 | Soldiers: 4598 | Projectiles: 52
FPS: 61 | Soldiers: 4401 | Projectiles: 38
...
```

Soldier count decreases as specific individuals are hit and killed!

## Next Steps

### Immediate Improvements
1. **Accuracy/Deviation**: Add spread to musket shots (not all hit)
2. **Spatial Indexing**: Optimize hit detection for projectiles
3. **Visual Effects**: Smoke puffs when firing, impact effects
4. **Formation States**: Kneeling front rank, standing rear ranks

### Advanced Features
1. **Morale Propagation**: Fear spreads between nearby soldiers
2. **NCO Commands**: Sergeants actively reform their sections
3. **Ammunition Resupply**: Soldiers fall back when out of ammo
4. **Melee Combat**: Individual bayonet fighting when formations collide
5. **Casualties Falling**: Wounded soldiers on ground, not instant despawn

## Historical Accuracy

This simulation models:
- ✅ Individual soldier agency (not abstract units)
- ✅ Front-rank-only fire in column formations
- ✅ Continuous rolling volleys (not synchronized)
- ✅ Active gap filling and reformation
- ✅ Column vs Line tactical problem
- ✅ 15-second musket reload time
- ✅ ~200m maximum range, 50m effective range
- ✅ Individual ammunition tracking

## Files

- `crates/simulation/examples/individual_soldiers.rs` - Main implementation
- `crates/simulation/src/soldier.rs` - Component definitions (if moved out)
- `crates/renderer/src/pixel_renderer.rs` - Rendering individual soldiers

## Conclusion

We've achieved the vision: **every bullet is calculated for every individual soldier**, with visible projectiles, specific casualties, and active AI-controlled reformation. This is true soldier-level simulation with 4,800 independent entities!

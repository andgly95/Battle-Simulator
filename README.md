# Battle Simulator

A high-performance, historically accurate battle simulator built in Rust. Starting with the Napoleonic era with plans to expand through WW1 and beyond.

## Features

- **High Performance**: Handle 10,000+ units at 60+ FPS using ECS architecture
- **Historical Accuracy**: Authentic unit capabilities, tactics, and combat mechanics based on historical sources
- **Cross-Platform**: Runs on Windows, Linux, and macOS
- **Extensible Design**: Modular era-based plugin system for different time periods
- **Advanced AI**: Hierarchical command structure with realistic tactical decision-making
- **Detailed Combat**: Realistic ballistics, morale, fatigue, and formation mechanics

## Current Status

🚧 **In Active Development** 🚧

Currently in the architecture and planning phase. See [ARCHITECTURE.md](ARCHITECTURE.md) for the complete technical design.

## Planned Eras

### Phase 1: Napoleonic Wars (1803-1815)
- Line, column, and square formations
- Musket and artillery combat
- Cavalry charges
- Morale and routing mechanics
- Historical scenarios: Waterloo, Austerlitz, Borodino

### Phase 2: World War 1 (1914-1918)
- Trench warfare
- Machine guns and artillery barrages
- Gas warfare
- Tank mechanics
- Historical scenarios: Somme, Verdun, Passchendaele

## Technology Stack

- **Language**: Rust (for performance and safety)
- **ECS**: bevy_ecs or specs
- **Graphics**: wgpu (cross-platform GPU rendering)
- **Math**: glam (SIMD-optimized)
- **Parallelism**: rayon

## Architecture Highlights

### Entity Component System (ECS)
Data-oriented design for maximum performance with thousands of entities:
- Cache-friendly memory layout
- Parallel system execution
- Flexible entity composition

### Spatial Partitioning
- Uniform grid for O(1) insertion and fast neighbor queries
- Only process combat between units in adjacent cells
- Scales to massive battles efficiently

### Accurate Combat Model
- Historical weapon statistics
- Formation effectiveness modifiers
- Morale and fatigue simulation
- Environmental factors (weather, terrain, visibility)
- Realistic casualty rates validated against historical battles

### Multi-threaded Simulation
- Parallel combat resolution
- Parallel AI decision making
- Lock-free spatial queries
- Scales across CPU cores

## Performance Targets

| Scenario | Units | Target FPS | Memory |
|----------|-------|------------|--------|
| Small Skirmish | 500 | 120+ | <500MB |
| Medium Battle | 2,000 | 90+ | <1GB |
| Large Battle | 5,000 | 60+ | <2GB |
| Epic Battle | 10,000 | 30+ | <3GB |

## Project Structure

```
battle-simulator/
├── crates/
│   ├── core/              # Core ECS and engine
│   ├── simulation/        # Battle simulation logic
│   ├── era_napoleonic/    # Napoleonic era implementation
│   ├── era_ww1/           # WW1 era implementation
│   ├── renderer/          # Graphics rendering
│   └── cli/               # Command-line interface
├── data/                  # Game data (units, weapons, scenarios)
├── docs/                  # Documentation
└── tests/                 # Integration tests
```

## Building

```bash
# Clone the repository
git clone https://github.com/yourusername/battle-simulator.git
cd battle-simulator

# Build the project
cargo build --release

# Run tests
cargo test

# Run benchmarks
cargo bench
```

## Usage

```bash
# Run a historical scenario
cargo run --release -- scenario waterloo

# Run a custom battle
cargo run --release -- custom --units-per-side 5000

# Benchmark performance
cargo run --release -- bench --units 10000
```

## Documentation

- [ARCHITECTURE.md](ARCHITECTURE.md) - Complete technical architecture
- [docs/combat_mechanics.md](docs/combat_mechanics.md) - Detailed combat system documentation
- [docs/ai_design.md](docs/ai_design.md) - AI system design
- [docs/era_guide.md](docs/era_guide.md) - Guide for implementing new eras

## Historical Accuracy

The simulator is based on extensive historical research:

- Nafziger Order of Battle Collection
- "Battle Tactics of Napoleon and His Enemies" - Brent Nosworthy
- "The Anatomy of Victory" - Rory Muir
- Period drill manuals and tactical treatises
- Historical casualty statistics and battle reports

Combat results are validated against historical battle outcomes to ensure accuracy.

## Contributing

Contributions are welcome! Areas of interest:
- Historical unit data and order of battle
- Combat mechanics refinement
- AI improvements
- New era implementations
- Performance optimizations

## License

MIT License - See LICENSE file for details

## Roadmap

- [x] Architecture design
- [ ] Core ECS implementation
- [ ] Spatial partitioning system
- [ ] Basic movement and pathfinding
- [ ] Napoleonic combat mechanics
- [ ] Morale system
- [ ] Formation system
- [ ] Tactical AI
- [ ] Historical scenarios
- [ ] WW1 era plugin
- [ ] Graphics rendering
- [ ] Campaign mode

## Contact

For questions or suggestions, please open an issue on GitHub.

---

*"In war, the moral is to the physical as three is to one."* - Napoleon Bonaparte

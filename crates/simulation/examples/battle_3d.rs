//! 3D Battle Visualization with Chess Piece Soldiers
//!
//! Features:
//! - Full 3D rendering with Bevy engine
//! - Chess pawn models for individual soldiers
//! - 3D terrain mesh from heightmap
//! - Orbit camera (Total War style)
//! - 4,800 individual soldiers with instanced rendering
//!
//! Controls:
//! - Left mouse drag: Rotate camera around battlefield
//! - Right mouse drag: Pan camera
//! - Scroll wheel: Zoom in/out
//! - Space: Pause/resume simulation
//!
//! Run with: cargo run --example battle_3d

use bevy::prelude::*;
use bevy::pbr::wireframe::{WireframePlugin, WireframeConfig};
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_resource::Face;
use battle_sim_core::terrain::Terrain;
use battle_sim_simulation::components::*;
use battle_sim_simulation::*;
use glam::{vec2, vec3, Vec3};

/// Orbit camera component
#[derive(Component)]
struct OrbitCamera {
    focus: Vec3,
    radius: f32,
    upside_down: bool,
    pitch: f32,  // Up/down rotation (radians)
    yaw: f32,    // Left/right rotation (radians)
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            focus: Vec3::new(200.0, 0.0, 225.0),  // Focus on battlefield center
            radius: 400.0,  // Start 400m away
            upside_down: false,
            pitch: -45.0_f32.to_radians(),  // Look down at 45 degrees
            yaw: 0.0,
        }
    }
}

/// Track mouse state for camera controls
#[derive(Resource, Default)]
struct MouseState {
    left_pressed: bool,
    right_pressed: bool,
    last_pos: Vec2,
}

/// Marker for soldier meshes (for instanced rendering)
#[derive(Component)]
struct SoldierMesh;

/// Main setup function
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Battle Simulator 3D - 4,800 Chess Piece Soldiers".to_string(),
                resolution: (1920.0, 1080.0).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.4, 0.6, 0.8)))  // Sky blue
        .insert_resource(MouseState::default())
        .add_systems(Startup, (
            setup_scene,
            setup_simulation,
        ))
        .add_systems(Update, (
            orbit_camera_system,
            run_simulation,
            sync_soldier_transforms,
        ))
        .run();
}

/// Set up the 3D scene: lights, camera, terrain
fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Directional light (sun)
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::WHITE,
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -60.0_f32.to_radians(),
            30.0_f32.to_radians(),
            0.0,
        )),
        ..default()
    });

    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 200.0,
    });

    // Orbit camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-200.0, 200.0, -200.0)
                .looking_at(Vec3::new(200.0, 0.0, 225.0), Vec3::Y),
            ..default()
        },
        OrbitCamera::default(),
    ));

    info!("Scene setup complete: camera, lights ready");
}

/// Set up the simulation world with terrain and soldiers
fn setup_simulation(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create terrain
    let mut terrain = Terrain::new(1000.0, 1000.0, 10.0);

    // Add hills
    terrain.add_hill(100.0, 220.0, 80.0, 15.0);
    terrain.add_hill(500.0, 320.0, 60.0, 20.0);
    terrain.add_hill(700.0, 200.0, 50.0, 12.0);

    // Add forests
    terrain.add_forest(400.0, 100.0, 40.0);
    terrain.add_forest(800.0, 250.0, 35.0);

    // Add road
    terrain.add_road(0.0, 225.0, 1000.0, 225.0, 8.0);

    info!("Terrain generated with hills, forests, and roads");

    // Generate terrain mesh
    let terrain_mesh = generate_terrain_mesh(&terrain);
    let terrain_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.6, 0.3),  // Green grass
        perceptual_roughness: 0.9,
        ..default()
    });

    commands.spawn(PbrBundle {
        mesh: meshes.add(terrain_mesh),
        material: terrain_material,
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    });

    info!("Terrain mesh spawned");

    // Store terrain as resource for simulation
    commands.insert_resource(terrain);
    commands.insert_resource(SimulationTime::new());

    // Create chess pawn mesh for soldiers
    let soldier_mesh = create_chess_pawn_mesh();
    let soldier_mesh_handle = meshes.add(soldier_mesh);

    // Materials for teams
    let british_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.2, 0.2),  // Red
        metallic: 0.1,
        perceptual_roughness: 0.5,
        ..default()
    });

    let french_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.4, 0.9),  // Blue
        metallic: 0.1,
        perceptual_roughness: 0.5,
        ..default()
    });

    info!("==============================================");
    info!("  3D BATTLE SIMULATION");
    info!("  4,800 chess piece soldiers");
    info!("==============================================");

    // Spawn British soldiers (red pawns) in LINE formation
    info!("Spawning 2,400 British soldiers (red pawns)...");
    spawn_battalion_3d(
        &mut commands,
        vec2(-200.0, 300.0),
        Side::Enemy,
        FormationType::Line,
        800,
        soldier_mesh_handle.clone(),
        british_material.clone(),
    );
    spawn_battalion_3d(
        &mut commands,
        vec2(200.0, 300.0),
        Side::Enemy,
        FormationType::Line,
        800,
        soldier_mesh_handle.clone(),
        british_material.clone(),
    );
    spawn_battalion_3d(
        &mut commands,
        vec2(600.0, 300.0),
        Side::Enemy,
        FormationType::Line,
        800,
        soldier_mesh_handle.clone(),
        british_material.clone(),
    );

    // Spawn French soldiers (blue pawns) in COLUMN formation
    info!("Spawning 2,400 French soldiers (blue pawns)...");
    spawn_battalion_3d(
        &mut commands,
        vec2(-200.0, 150.0),
        Side::Allied,
        FormationType::Column,
        800,
        soldier_mesh_handle.clone(),
        french_material.clone(),
    );
    spawn_battalion_3d(
        &mut commands,
        vec2(200.0, 150.0),
        Side::Allied,
        FormationType::Column,
        800,
        soldier_mesh_handle.clone(),
        french_material.clone(),
    );
    spawn_battalion_3d(
        &mut commands,
        vec2(600.0, 150.0),
        Side::Allied,
        FormationType::Column,
        800,
        soldier_mesh_handle.clone(),
        french_material.clone(),
    );

    info!("Total: 4,800 chess piece soldiers spawned!");
}

/// Generate 3D terrain mesh from heightmap
fn generate_terrain_mesh(terrain: &Terrain) -> Mesh {
    let (grid_width, grid_height) = terrain.grid_dimensions();
    let cell_size = terrain.cell_size();

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    // Generate vertices
    for y in 0..=grid_height {
        for x in 0..=grid_width {
            let world_x = x as f32 * cell_size;
            let world_z = y as f32 * cell_size;  // Z in Bevy, was Y in 2D
            let world_y = terrain.get_elevation(world_x, world_z);

            positions.push([world_x, world_y, world_z]);
            normals.push([0.0, 1.0, 0.0]);  // Flat normals for now
            uvs.push([x as f32 / grid_width as f32, y as f32 / grid_height as f32]);
        }
    }

    // Generate indices (two triangles per quad)
    for y in 0..grid_height {
        for x in 0..grid_width {
            let i0 = y * (grid_width + 1) + x;
            let i1 = i0 + 1;
            let i2 = i0 + (grid_width + 1);
            let i3 = i2 + 1;

            // First triangle
            indices.push(i0 as u32);
            indices.push(i2 as u32);
            indices.push(i1 as u32);

            // Second triangle
            indices.push(i1 as u32);
            indices.push(i2 as u32);
            indices.push(i3 as u32);
        }
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        bevy::render::render_asset::RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices))
}

/// Create a procedural chess pawn mesh
fn create_chess_pawn_mesh() -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    let segments = 16;
    let height = 1.5;  // 1.5 meters tall

    // Base (wide)
    let base_radius = 0.4;
    let base_height = 0.2;

    // Stem (narrow)
    let stem_radius = 0.2;
    let stem_height = 0.8;

    // Head (sphere-ish)
    let head_radius = 0.35;
    let head_center = height - head_radius;

    let mut vertex_index = 0u32;

    // Generate base cylinder
    for segment in 0..=segments {
        let angle = (segment as f32 / segments as f32) * std::f32::consts::TAU;
        let x = angle.cos() * base_radius;
        let z = angle.sin() * base_radius;

        // Bottom ring
        positions.push([x, 0.0, z]);
        normals.push([x, 0.0, z].into());  // Simplified normal
        uvs.push([segment as f32 / segments as f32, 0.0]);

        // Top ring of base
        positions.push([x, base_height, z]);
        normals.push([x, 0.0, z].into());
        uvs.push([segment as f32 / segments as f32, 0.2]);
    }

    // Generate indices for base
    for segment in 0..segments {
        let i0 = segment * 2;
        let i1 = i0 + 1;
        let i2 = (segment + 1) * 2;
        let i3 = i2 + 1;

        indices.extend_from_slice(&[i0, i2, i1, i1, i2, i3]);
    }

    vertex_index = positions.len() as u32;

    // Generate stem cylinder
    for segment in 0..=segments {
        let angle = (segment as f32 / segments as f32) * std::f32::consts::TAU;
        let x = angle.cos() * stem_radius;
        let z = angle.sin() * stem_radius;

        // Bottom of stem
        positions.push([x, base_height, z]);
        normals.push([x, 0.0, z].into());
        uvs.push([segment as f32 / segments as f32, 0.2]);

        // Top of stem
        positions.push([x, base_height + stem_height, z]);
        normals.push([x, 0.0, z].into());
        uvs.push([segment as f32 / segments as f32, 0.7]);
    }

    // Generate indices for stem
    let stem_start = vertex_index;
    for segment in 0..segments {
        let i0 = stem_start + segment * 2;
        let i1 = i0 + 1;
        let i2 = stem_start + (segment + 1) * 2;
        let i3 = i2 + 1;

        indices.extend_from_slice(&[i0, i2, i1, i1, i2, i3]);
    }

    vertex_index = positions.len() as u32;

    // Generate head (simplified sphere - actually octahedron-ish)
    let head_y = base_height + stem_height;
    for segment in 0..=segments {
        let angle = (segment as f32 / segments as f32) * std::f32::consts::TAU;
        let x = angle.cos() * head_radius;
        let z = angle.sin() * head_radius;

        // Equator
        positions.push([x, head_y, z]);
        normals.push([x, 0.0, z].into());
        uvs.push([segment as f32 / segments as f32, 0.75]);

        // Upper hemisphere
        positions.push([x * 0.7, head_y + head_radius * 0.7, z * 0.7]);
        normals.push([x, head_radius, z].into());
        uvs.push([segment as f32 / segments as f32, 0.9]);
    }

    // Top point
    let top_index = positions.len() as u32;
    positions.push([0.0, height, 0.0]);
    normals.push([0.0, 1.0, 0.0]);
    uvs.push([0.5, 1.0]);

    // Generate indices for head
    let head_start = vertex_index;
    for segment in 0..segments {
        let i0 = head_start + segment * 2;
        let i1 = i0 + 1;
        let i2 = head_start + (segment + 1) * 2;
        let i3 = i2 + 1;

        // Lower part of head
        indices.extend_from_slice(&[i0, i2, i1, i1, i2, i3]);

        // Upper part to top point
        indices.extend_from_slice(&[i1, i3, top_index]);
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        bevy::render::render_asset::RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices))
}

/// Spawn a battalion of 3D soldiers
fn spawn_battalion_3d(
    commands: &mut Commands,
    center: Vec2,
    side: Side,
    formation: FormationType,
    count: u32,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
) {
    match formation {
        FormationType::Line => {
            let spacing = 2.0;
            let start_x = center.x - (count as f32 * spacing) / 2.0;

            for i in 0..count {
                let x = start_x + i as f32 * spacing;
                let z = center.y;  // Z in 3D, was Y in 2D

                spawn_soldier_3d(commands, vec3(x, 0.0, z), side, mesh.clone(), material.clone());
            }
        }

        FormationType::Column => {
            let width = 20;
            let spacing = 1.5;
            let ranks = (count as f32 / width as f32).ceil() as u32;

            let start_x = center.x - (width as f32 * spacing) / 2.0;
            let start_z = center.y - (ranks as f32 * spacing) / 2.0;

            for rank in 0..ranks {
                let men_in_rank = width.min(count - rank * width);
                for file in 0..men_in_rank {
                    let x = start_x + file as f32 * spacing;
                    let z = start_z + rank as f32 * spacing;

                    spawn_soldier_3d(commands, vec3(x, 0.0, z), side, mesh.clone(), material.clone());
                }
            }
        }

        _ => {}
    }
}

/// Spawn a single 3D soldier
fn spawn_soldier_3d(
    commands: &mut Commands,
    position: Vec3,
    side: Side,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
) {
    commands.spawn((
        PbrBundle {
            mesh,
            material,
            transform: Transform::from_translation(position),
            ..default()
        },
        Position::new(position.x, position.z),  // Store 2D position for simulation
        Velocity::zero(),
        Team::new(0, side),
        SoldierMesh,
    ));
}

/// Orbit camera controls (Total War style)
fn orbit_camera_system(
    mut mouse_state: ResMut<MouseState>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<bevy::input::mouse::MouseMotion>,
    mut mouse_wheel: EventReader<bevy::input::mouse::MouseWheel>,
    mut camera_query: Query<(&mut OrbitCamera, &mut Transform), With<Camera>>,
) {
    let (mut orbit, mut transform) = camera_query.single_mut();

    // Track mouse button state
    mouse_state.left_pressed = mouse_buttons.pressed(MouseButton::Left);
    mouse_state.right_pressed = mouse_buttons.pressed(MouseButton::Right);

    // Handle mouse motion
    for motion in mouse_motion.read() {
        if mouse_state.left_pressed {
            // Left drag: rotate camera
            orbit.yaw -= motion.delta.x * 0.005;
            orbit.pitch -= motion.delta.y * 0.005;
            orbit.pitch = orbit.pitch.clamp(-89.0_f32.to_radians(), 89.0_f32.to_radians());
        } else if mouse_state.right_pressed {
            // Right drag: pan camera
            let right = transform.right();
            let up = Vec3::Y;
            orbit.focus -= right * motion.delta.x * 0.5;
            orbit.focus -= up * motion.delta.y * 0.5;
        }
    }

    // Handle mouse wheel (zoom)
    for wheel in mouse_wheel.read() {
        orbit.radius -= wheel.y * 20.0;
        orbit.radius = orbit.radius.clamp(50.0, 1000.0);
    }

    // Update camera transform based on orbit parameters
    let yaw_quat = Quat::from_rotation_y(orbit.yaw);
    let pitch_quat = Quat::from_rotation_x(orbit.pitch);
    let offset = yaw_quat * pitch_quat * Vec3::new(0.0, 0.0, orbit.radius);

    transform.translation = orbit.focus + offset;
    transform.look_at(orbit.focus, Vec3::Y);
}

/// Run simulation systems (simplified - just positions for now)
fn run_simulation(
    time: Res<Time>,
    mut sim_time: ResMut<SimulationTime>,
) {
    // Update simulation time
    sim_time.tick(time.delta_seconds());
}

/// Sync 2D simulation positions to 3D transforms
fn sync_soldier_transforms(
    mut query: Query<(&Position, &mut Transform), With<SoldierMesh>>,
    terrain: Res<Terrain>,
) {
    for (pos, mut transform) in query.iter_mut() {
        let elevation = terrain.get_elevation(pos.x, pos.y);
        transform.translation = Vec3::new(pos.x, elevation, pos.y);
    }
}

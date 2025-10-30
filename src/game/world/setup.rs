use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use super::state::PlayerCount;

pub fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Note: Camera is now spawned per-player in the rendering system
    // when the local player entity is replicated

    // Add lighting
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, 0.5, 0.0)),
    ));

    // Add ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 300.0,
        affects_lightmapped_meshes: false,
    });

    // Spawn environment decoration
    commands.spawn((
        SceneRoot(asset_server.load("models/environment.glb#Scene0")),
        Transform::from_xyz(10.0, 0.0, 10.0)
            .with_scale(Vec3::splat(1.0)),
    ));

    // Ground plane - must have RigidBody::Fixed to be a true static collider
    // Positioned at y=0.0, collider extends from y=-0.1 to y=0.1
    let ground_entity = commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(50.0)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.7, 0.3),
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
        GlobalTransform::default(),
        RigidBody::Fixed,  // CRITICAL: Marks this as a static, immovable collider
        Collider::cuboid(50.0, 0.1, 50.0),
        Friction {
            coefficient: 0.7,
            combine_rule: CoefficientCombineRule::Average,
        },
        Restitution {
            coefficient: 0.0,
            combine_rule: CoefficientCombineRule::Min,
        },
    )).id();

    println!("[SETUP] Ground entity {:?} created at y=0.0 with 50x50 collider", ground_entity);
}

pub fn init_server_state(mut commands: Commands) {
    commands.insert_resource(PlayerCount(0));
    println!("[SERVER] Physics initialized with default configuration");
}

// Server-only world setup (no rendering, just physics)
pub fn setup_server_world(mut commands: Commands) {
    // Ground plane - static collider only (no mesh/material for headless server)
    // Must have RigidBody::Fixed to be a true static collider
    let ground_entity = commands.spawn((
        Transform::from_xyz(0.0, 0.0, 0.0),
        GlobalTransform::default(),
        RigidBody::Fixed,  // CRITICAL: Marks this as a static, immovable collider
        Collider::cuboid(50.0, 0.1, 50.0),
        Friction {
            coefficient: 0.7,
            combine_rule: CoefficientCombineRule::Average,
        },
        Restitution {
            coefficient: 0.0,
            combine_rule: CoefficientCombineRule::Min,
        },
    )).id();

    println!("[SERVER] Ground collider entity {:?} created at y=0.0 (50x50x0.2)", ground_entity);
}

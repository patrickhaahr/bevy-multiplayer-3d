use bevy::prelude::*;

use super::state::PlayerCount;

pub fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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

    // Ground plane (moved down to -1.0 so models are visible)
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(10.0)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.7, 0.3),
            ..default()
        })),
        Transform::from_xyz(0.0, -1.0, 0.0),
    ));
}

pub fn init_server_state(mut commands: Commands) {
    commands.insert_resource(PlayerCount(0));
}

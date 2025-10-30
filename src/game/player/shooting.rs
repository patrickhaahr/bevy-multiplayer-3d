use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use super::camera_controller::CameraController;
use super::components::TracerSpawnSpot;
use crate::game::shooting::BulletTracer;
use crate::network::protocol::{ShootEvent, Player, Enemy};
use crate::network::client::LocalClientId;

pub fn handle_shooting(
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    mut shoot_writer: MessageWriter<ShootEvent>,
    rapier_context: ReadRapierContext,
    camera_query: Query<(&Camera, &GlobalTransform), With<CameraController>>,
    window_query: Query<&Window>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    spawn_spot_query: Query<&GlobalTransform, With<TracerSpawnSpot>>,
    player_query: Query<(Entity, &Player)>,
    enemy_query: Query<&Enemy>,
    local_client_id: Res<LocalClientId>,
) {
    if !mouse_input.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = window_query.single() else {
        return;
    };

    let Ok((camera, camera_global_transform)) = camera_query.single() else {
        return;
    };

    let Ok(spawn_spot) = spawn_spot_query.single() else {
        return;
    };

    let Ok(rapier_context) = rapier_context.single() else {
        return;
    };

    let Ok(ray) = camera.viewport_to_world(
        camera_global_transform,
        Vec2::new(window.width() / 2., window.height() / 2.),
    ) else {
        return;
    };

    // Send shoot event to server
    shoot_writer.write(ShootEvent {
        origin: ray.origin,
        direction: *ray.direction,
    });

    // Find the local player entity to exclude from raycast
    let local_player_entity = player_query
        .iter()
        .find(|(_, player)| player.id == local_client_id.0)
        .map(|(entity, _)| entity);

    // Also perform local raycast for immediate visual feedback
    let filter = if let Some(local_entity) = local_player_entity {
        QueryFilter::default().exclude_rigid_body(local_entity)
    } else {
        QueryFilter::default()
    };

    rapier_context.with_query_pipeline(filter, |query_pipeline| {
        if let Some((entity, toi)) = query_pipeline.cast_ray(
            ray.origin,
            *ray.direction,
            f32::MAX,
            true,
        ) {
            let hit_position = ray.origin + *ray.direction * toi;

            // Log only if we hit a player or enemy
            if player_query.get(entity).is_ok() {
                println!("[CLIENT] Hit player at distance {:.2}m", toi);
            } else if enemy_query.get(entity).is_ok() {
                println!("[CLIENT] Hit enemy at distance {:.2}m", toi);
            }

            // Spawn tracer for visual feedback
            let tracer_material = StandardMaterial {
                base_color: Color::srgb(1., 1., 0.),
                unlit: true,
                ..default()
            };

            commands.spawn((
                Transform::from_translation(Vec3::splat(f32::MAX)),
                GlobalTransform::default(),
                Mesh3d(meshes.add(Cuboid::from_size(Vec3::new(0.1, 0.1, 1.0)))),
                MeshMaterial3d(materials.add(tracer_material)),
                BulletTracer::new(spawn_spot.translation(), hit_position, 400.),
            ));
        }
    });
}

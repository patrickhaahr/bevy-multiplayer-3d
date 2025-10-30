use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use super::camera_controller::CameraController;
use super::components::TracerSpawnSpot;
use crate::game::shooting::BulletTracer;
use crate::network::protocol::ShootEvent;

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

    println!("[CLIENT] Mouse clicked!");

    let Ok(ray) = camera.viewport_to_world(
        camera_global_transform,
        Vec2::new(window.width() / 2., window.height() / 2.),
    ) else {
        println!("[CLIENT] Failed to create viewport ray!");
        return;
    };

    println!(
        "[CLIENT] Ray origin: {:?}, direction: {:?}",
        ray.origin, ray.direction
    );

    // Send shoot event to server
    shoot_writer.write(ShootEvent {
        origin: ray.origin,
        direction: *ray.direction,
    });
    println!("[CLIENT] Sent ShootEvent to server");

    // Also perform local raycast for immediate visual feedback
    let filter = QueryFilter::default();

    rapier_context.with_query_pipeline(filter, |query_pipeline| {
        if let Some((entity, toi)) = query_pipeline.cast_ray(
            ray.origin,
            *ray.direction,
            f32::MAX,
            true,
        ) {
            println!("[CLIENT] Local hit entity {:?} at distance {}", entity, toi);

            let hit_position = ray.origin + *ray.direction * toi;
            println!(
                "[CLIENT] Spawning tracer from {:?} to {:?}",
                spawn_spot.translation(),
                hit_position
            );

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
        } else {
            println!("[CLIENT] No local hit detected!");
        }
    });
}

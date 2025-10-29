use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use super::camera_controller::CameraController;
use super::components::TracerSpawnSpot;
use crate::game::shooting::BulletTracer;

pub fn handle_shooting(
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
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

    println!("SHOOT: Mouse clicked!");

    let Ok(ray) = camera.viewport_to_world(
        camera_global_transform,
        Vec2::new(window.width() / 2., window.height() / 2.),
    ) else {
        println!("SHOOT: Failed to create viewport ray!");
        return;
    };

    println!(
        "SHOOT: Ray origin: {:?}, direction: {:?}",
        ray.origin, ray.direction
    );

    let filter = QueryFilter::default();

    rapier_context.with_query_pipeline(filter, |query_pipeline| {
        if let Some((entity, toi)) = query_pipeline.cast_ray(
            ray.origin,
            *ray.direction,
            f32::MAX,
            true,
        ) {
            println!("SHOOT: Hit entity {:?} at distance {}", entity, toi);

            let hit_position = ray.origin + *ray.direction * toi;
            println!(
                "SHOOT: Spawning tracer from {:?} to {:?}",
                spawn_spot.translation(),
                hit_position
            );

            // Spawn tracer
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
            println!("SHOOT: No hit detected!");
        }
    });
}

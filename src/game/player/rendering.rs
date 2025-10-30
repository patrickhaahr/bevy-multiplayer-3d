use bevy::prelude::*;

use super::components::{GunModel, RenderedPlayer, TracerSpawnSpot};
use super::camera_controller::CameraController;
use crate::network::protocol::{Player, PlayerPosition, PlayerRotation};
use crate::network::client::LocalClientId;

pub fn render_replicated_players(
    mut commands: Commands,
    players: Query<(Entity, &Player, &PlayerPosition, &PlayerRotation), Without<RenderedPlayer>>,
    asset_server: Res<AssetServer>,
    local_client_id: Res<LocalClientId>,
) {
    let client_id = local_client_id.0;
    
    for (entity, player, pos, rotation) in players.iter() {
        println!(
            "Rendering player {} at ({}, {}, {}) with rotation (yaw: {}, pitch: {})",
            player.id, pos.x, pos.y, pos.z, rotation.yaw, rotation.pitch
        );

        let is_local_player = player.id == client_id;

        // Add base rendering components
        commands.entity(entity).insert((
            RenderedPlayer,
            Transform::from_xyz(pos.x, pos.y, pos.z),
            GlobalTransform::default(),
            Visibility::Inherited,
            InheritedVisibility::default(),
            ViewVisibility::default(),
        ));

        if is_local_player {
            // For local player: spawn first-person camera with gun
            println!("Setting up first-person camera for local player {}", player.id);
            
            let fov: f32 = 103.0_f32.to_radians();
            
            // Spawn camera as child of player entity
            let camera_entity = commands.spawn((
                Camera3d::default(),
                Projection::Perspective(PerspectiveProjection { fov, ..default() }),
                Transform::from_xyz(0.0, 1.6, 0.0), // Eye height above player origin
                GlobalTransform::default(),
                CameraController {
                    sensitivity: 0.035,
                    rotation: Vec2::new(rotation.pitch, rotation.yaw),
                    rotation_lock: 88.0,
                },
            )).id();
            
            // Load gun model as child of camera
            let gun_model: Handle<Scene> = asset_server.load("models/gun.glb#Scene0");
            let gun_entity = commands.spawn((
                SceneRoot(gun_model),
                Transform::from_xyz(0.3, -0.2, -0.3),
                GlobalTransform::default(),
                GunModel,
            )).id();
            
            // Tracer spawn spot as child of camera
            let tracer_spawn_entity = commands.spawn((
                Transform::from_xyz(0.3, -0.2, -0.5),
                GlobalTransform::default(),
                TracerSpawnSpot,
            )).id();
            
            // Build hierarchy: player -> camera -> (gun, tracer spawn)
            commands.entity(camera_entity).set_parent_in_place(entity);
            commands.entity(gun_entity).set_parent_in_place(camera_entity);
            commands.entity(tracer_spawn_entity).set_parent_in_place(camera_entity);
            
        } else {
            // For remote players: spawn third-person model as child entity
            println!("Loading third-person model for remote player {}", player.id);
            let player_model: Handle<Scene> = asset_server.load("models/player.glb#Scene0");
            
            let model_entity = commands.spawn((
                SceneRoot(player_model),
                Transform::from_xyz(0.0, -1.5, 0.0)
                    .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
                GlobalTransform::default(),
            )).id();
            
            // Load gun model for remote player
            let gun_model: Handle<Scene> = asset_server.load("models/gun.glb#Scene0");
            let gun_entity = commands.spawn((
                SceneRoot(gun_model),
                Transform::from_xyz(0.5, 0.8, 0.3)  // Position gun relative to player model
                    .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
                GlobalTransform::default(),
            )).id();
            
            // Make model and gun children of player entity
            commands.entity(model_entity).set_parent_in_place(entity);
            commands.entity(gun_entity).set_parent_in_place(entity);
        }
    }
}

pub fn sync_remote_player_rotation(
    mut players: Query<(&Player, &PlayerRotation, &mut Transform), With<RenderedPlayer>>,
    local_client_id: Res<LocalClientId>,
) {
    let client_id = local_client_id.0;
    
    for (player, rotation, mut transform) in players.iter_mut() {
        // Only update remote players (not the local player)
        if player.id != client_id {
            // Apply yaw rotation to the player entity (makes them face the right direction)
            let y_quat = Quat::from_axis_angle(Vec3::Y, rotation.yaw.to_radians());
            transform.rotation = y_quat;
        }
    }
}

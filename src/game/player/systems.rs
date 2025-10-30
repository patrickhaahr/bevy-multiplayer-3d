use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon::shared::backend::connected_client::NetworkId;
use bevy_replicon_renet::renet::ServerEvent;

use crate::game::world::state::PlayerCount;
use crate::game::player::components::PlayerPhysicsBundle;
use crate::network::protocol::{Player, PlayerPosition, PlayerRotation, Health, RotationInput, MovementInput, ShootEvent, Enemy};

pub fn spawn_players_system(
    mut commands: Commands,
    mut server_events: MessageReader<ServerEvent>,
    mut player_count: ResMut<PlayerCount>,
) {
    for event in server_events.read() {
        if let ServerEvent::ClientConnected { client_id } = event {
            // Position players in a circle around the origin
            let angle = player_count.0 as f32 * std::f32::consts::TAU / 4.0; // Distribute evenly
            let radius = 3.0;
            let x = angle.cos() * radius;
            let z = angle.sin() * radius;

            let color_index = player_count.0 as u8;
            player_count.0 += 1;

            // Spawn player entity with physics (server-side)
            // Spawn at y=5.0 so we can see them fall and collide with ground
            let spawn_y = 5.0;
            let player_entity = commands
                .spawn((
                    Player {
                        id: *client_id,
                        color_index,
                    },
                    PlayerPosition { x, y: spawn_y, z },
                    PlayerRotation { yaw: 0.0, pitch: 0.0 },
                    Health { current: 100.0, max: 100.0 },
                    Transform::from_xyz(x, spawn_y, z),
                    GlobalTransform::default(),
                    PlayerPhysicsBundle::default(),
                    Replicated,
                ))
                .id();

            println!(
                "Spawned player entity {:?} for client {} at position ({}, {}, {})",
                player_entity, client_id, x, spawn_y, z
            );
        }
    }
}

pub fn handle_rotation_input(
    mut rotation_inputs: MessageReader<FromClient<RotationInput>>,
    mut players: Query<(&Player, &mut PlayerRotation)>,
    client_entities: Query<&NetworkId>,
) {
    for input in rotation_inputs.read() {
        let sender_entity = match input.client_id {
            ClientId::Client(entity) => entity,
            ClientId::Server => {
                // Skip server messages (shouldn't happen for client input)
                continue;
            }
        };

        // Get the NetworkId (u64) from the client entity
        let Ok(network_id) = client_entities.get(sender_entity) else {
            warn!("Received rotation input from unknown client entity {:?}", sender_entity);
            continue;
        };

        let client_id = network_id.get();

        // Find the player with matching id and update their rotation
        for (player, mut rotation) in players.iter_mut() {
            if player.id == client_id {
                rotation.yaw = input.message.yaw;
                rotation.pitch = input.message.pitch;
                break;
            }
        }
    }
}

pub fn handle_movement_input(
    mut movement_inputs: MessageReader<FromClient<MovementInput>>,
    mut players: Query<(&Player, &PlayerRotation, &mut Velocity)>,
    client_entities: Query<&NetworkId>,
) {
    const MOVE_SPEED: f32 = 5.0; // Units per second

    for input in movement_inputs.read() {
        let sender_entity = match input.client_id {
            ClientId::Client(entity) => entity,
            ClientId::Server => {
                continue;
            }
        };

        let Ok(network_id) = client_entities.get(sender_entity) else {
            warn!("Received movement input from unknown client entity {:?}", sender_entity);
            continue;
        };

        let client_id = network_id.get();

        // Find the player and update their velocity based on movement input
        for (player, rotation, mut velocity) in players.iter_mut() {
            if player.id == client_id {
                let yaw_radians = rotation.yaw.to_radians();

                // Calculate forward and right directions based on yaw
                let forward = Vec3::new(-yaw_radians.sin(), 0.0, -yaw_radians.cos());
                let right = Vec3::new(yaw_radians.cos(), 0.0, -yaw_radians.sin());

                // Calculate desired movement direction
                let movement_vector =
                    forward * input.message.forward +
                    right * input.message.right;

                // Normalize if moving diagonally to prevent faster movement
                let movement_direction = if movement_vector.length() > 0.0 {
                    movement_vector.normalize()
                } else {
                    Vec3::ZERO
                };

                // Set horizontal velocity (preserve vertical velocity for gravity/jumping)
                velocity.linvel.x = movement_direction.x * MOVE_SPEED;
                velocity.linvel.z = movement_direction.z * MOVE_SPEED;
                // Don't touch velocity.linvel.y - let physics/gravity handle it

                break;
            }
        }
    }
}

// Server-side system to sync physics Transform back to replicated PlayerPosition
pub fn sync_transform_to_position(
    mut players: Query<(&Transform, &mut PlayerPosition), With<Player>>,
) {
    for (transform, mut position) in players.iter_mut() {
        // Update replicated position from physics transform
        position.x = transform.translation.x;
        position.y = transform.translation.y;
        position.z = transform.translation.z;
    }
}

// Server-side system to handle shoot events from clients
pub fn handle_shoot_events(
    mut shoot_events: MessageReader<FromClient<ShootEvent>>,
    client_entities: Query<&NetworkId>,
    players: Query<(Entity, &Player)>,
    mut player_healths: Query<&mut Health, With<Player>>,
    enemies: Query<Entity, With<Enemy>>,
    rapier_context: ReadRapierContext,
) {
    const DAMAGE_PER_HIT: f32 = 25.0;
    
    for event in shoot_events.read() {
        let sender_entity = match event.client_id {
            ClientId::Client(entity) => entity,
            ClientId::Server => {
                continue;
            }
        };

        let Ok(network_id) = client_entities.get(sender_entity) else {
            warn!("Received shoot event from unknown client entity {:?}", sender_entity);
            continue;
        };

        let client_id = network_id.get();
        
        // Find the shooter's player entity
        let shooter_entity = players
            .iter()
            .find(|(_, player)| player.id == client_id)
            .map(|(entity, _)| entity);
        
        let Some(shooter_entity) = shooter_entity else {
            warn!("Received shoot event from client {} without a player", client_id);
            continue;
        };

        // Perform raycast on server
        let Ok(rapier_context) = rapier_context.single() else {
            warn!("No rapier context available");
            continue;
        };

        // Exclude the shooter's own collider from the raycast
        let filter = QueryFilter::default().exclude_rigid_body(shooter_entity);

        rapier_context.with_query_pipeline(filter, |query_pipeline| {
            if let Some((hit_entity, toi)) = query_pipeline.cast_ray(
                event.message.origin,
                event.message.direction,
                f32::MAX,
                true,
            ) {
                // Check if we hit an enemy
                if enemies.contains(hit_entity) {
                    println!("[SERVER] Client {} hit enemy {:?} at {:.2}m", 
                        client_id, hit_entity, toi);
                }
                // Check if we hit a player
                else if let Ok((hit_entity, hit_player)) = players.get(hit_entity) {
                    // Apply damage to the hit player
                    if let Ok(mut health) = player_healths.get_mut(hit_entity) {
                        health.current -= DAMAGE_PER_HIT;
                        
                        if health.current <= 0.0 {
                            health.current = 0.0;
                            println!("[SERVER] Client {} killed player {} at {:.2}m", 
                                client_id, hit_player.id, toi);
                            // TODO: Despawn player or trigger respawn
                        } else {
                            println!("[SERVER] Client {} hit player {} at {:.2}m (Health: {:.0}/{:.0})", 
                                client_id, hit_player.id, toi, health.current, health.max);
                        }
                    }
                }
            }
        });
    }
}

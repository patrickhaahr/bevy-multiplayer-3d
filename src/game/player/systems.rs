use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon::shared::backend::connected_client::NetworkId;
use bevy_replicon_renet::renet::ServerEvent;

use crate::game::world::state::PlayerCount;
use crate::network::protocol::{Player, PlayerPosition, PlayerRotation, RotationInput};

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

            // Spawn player entity (server only tracks data, no rendering)
            let player_entity = commands
                .spawn((
                    Player {
                        id: *client_id,
                        color_index,
                    },
                    PlayerPosition { x, y: 0.5, z },
                    PlayerRotation { yaw: 0.0, pitch: 0.0 },
                    Replicated,
                ))
                .id();

            println!(
                "Spawned player entity {:?} for client {} at position ({}, 0.5, {})",
                player_entity, client_id, x, z
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

use bevy::prelude::*;
use bevy_replicon::prelude::Replicated;
use bevy_replicon_renet::renet::ServerEvent;

use crate::game::world::state::PlayerCount;
use crate::network::protocol::{Player, PlayerPosition};

pub fn spawn_players_system(
    mut commands: Commands,
    mut server_events: EventReader<ServerEvent>,
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

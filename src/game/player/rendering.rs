use bevy::prelude::*;

use super::components::RenderedPlayer;
use crate::network::protocol::{Player, PlayerPosition};

pub fn render_replicated_players(
    mut commands: Commands,
    players: Query<(Entity, &Player, &PlayerPosition), Without<RenderedPlayer>>,
    asset_server: Res<AssetServer>,
) {
    for (entity, player, pos) in players.iter() {
        println!(
            "Rendering player {} at ({}, {}, {})",
            player.id, pos.x, pos.y, pos.z
        );

        // Load player model
        let player_model: Handle<Scene> = asset_server.load("models/player.glb#Scene0");

        // Add rendering components to the replicated entity
        commands.entity(entity).insert((
            RenderedPlayer,
            SceneRoot(player_model),
            Transform::from_xyz(pos.x, pos.y, pos.z),
        ));

        // Spawn gun as a child entity
        let gun_model: Handle<Scene> = asset_server.load("models/gun.glb#Scene0");
        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                SceneRoot(gun_model),
                Transform::from_xyz(0.3, 0.5, 0.3), // Position relative to player
            ));
        });
    }
}

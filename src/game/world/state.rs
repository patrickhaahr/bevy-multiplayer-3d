use bevy::prelude::*;

// Resource to track player count for positioning
#[derive(Resource)]
pub struct PlayerCount(pub u32);

use bevy::prelude::*;

// Re-export from network protocol for convenience
pub use crate::network::protocol::{Player, PlayerPosition};

// Marker component for rendered players on client
#[derive(Component)]
pub struct RenderedPlayer;

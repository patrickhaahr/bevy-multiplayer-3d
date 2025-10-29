use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// Network constants
pub const PORT: u16 = 5000;
pub const PROTOCOL_ID: u64 = 0;

// Replicated components
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Player {
    pub id: u64,
    pub color_index: u8,
}

#[derive(Component, Serialize, Deserialize)]
pub struct PlayerPosition {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

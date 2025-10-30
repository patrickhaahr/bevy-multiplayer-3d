use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// Network constants
pub const PORT: u16 = 5000;
pub const PROTOCOL_ID: u64 = 0;

// Client -> Server events
#[derive(Message, Serialize, Deserialize)]
pub struct RotationInput {
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Message, Serialize, Deserialize)]
pub struct MovementInput {
    pub forward: f32,  // positive = forward, negative = backward
    pub right: f32,    // positive = right, negative = left
}

// Replicated components
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Player {
    pub id: u64,
    pub color_index: u8,
}

// Server-only component to map client entities to player entities
#[derive(Component)]
pub struct ClientEntity(pub bevy::prelude::Entity);

#[derive(Component, Serialize, Deserialize)]
pub struct PlayerPosition {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Component, Serialize, Deserialize)]
pub struct PlayerRotation {
    pub yaw: f32,   // Horizontal rotation (Y-axis) in degrees
    pub pitch: f32, // Vertical rotation (X-axis) in degrees
}

// Enemy replicated components
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Enemy {
    pub id: u32,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct EnemyPosition {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

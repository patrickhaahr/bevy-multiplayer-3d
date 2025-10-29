use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct PlayerInput {
    // if positive x = forward, positive y = right, negative x = backward, negative y = left
    pub movement: Vec2,
}

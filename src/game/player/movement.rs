use super::{camera_controller::CameraController, input::PlayerInput};
use bevy::prelude::*;

pub fn update_movement_input(keys: Res<ButtonInput<KeyCode>>, mut input: ResMut<PlayerInput>) {
    input.movement = Vec2::ZERO;

    if keys.pressed(KeyCode::KeyW) {
        input.movement.x += 1.;
    }
    if keys.pressed(KeyCode::KeyS) {
        input.movement.x -= 1.;
    }
    if keys.pressed(KeyCode::KeyA) {
        input.movement.y -= 1.;
    }
    if keys.pressed(KeyCode::KeyD) {
        input.movement.y += 1.;
    }
}

// This will be called client-side to send movement input to server
// For now, we'll just prepare the structure
pub fn apply_local_movement(
    input: Res<PlayerInput>,
    camera_query: Query<&CameraController>,
) {
    // TODO: Send input to server via events/RPC
    // Server will validate and apply movement
    if let Ok(_camera) = camera_query.single() {
        if input.movement.length_squared() > 0.0 {
            // Movement input detected - in full implementation, send to server
            // For now, this is a placeholder for client-side input collection
        }
    }
}

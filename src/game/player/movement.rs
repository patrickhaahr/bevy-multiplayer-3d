use super::{camera_controller::CameraController, input::PlayerInput};
use bevy::prelude::*;
use crate::network::protocol::MovementInput;

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

// Called client-side to send movement input to server
pub fn apply_local_movement(
    input: Res<PlayerInput>,
    camera_query: Query<&CameraController>,
    mut movement_writer: MessageWriter<MovementInput>,
) {
    if let Ok(_camera) = camera_query.single() {
        // Always send movement input, even if zero (for stopping movement)
        movement_writer.write(MovementInput {
            forward: input.movement.x,
            right: input.movement.y,
        });
    }
}

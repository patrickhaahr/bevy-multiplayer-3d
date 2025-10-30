use bevy::{input::mouse::MouseMotion, prelude::*};

use crate::network::protocol::RotationInput;

#[derive(Component)]
pub struct CameraController {
    pub rotation: Vec2,
    pub rotation_lock: f32,
    pub sensitivity: f32,
}

pub fn update_camera_controller(
    mut mouse_motion: MessageReader<MouseMotion>,
    mut camera_query: Query<(&mut CameraController, &mut Transform)>,
    mut rotation_writer: MessageWriter<RotationInput>,
) {
    if let Ok((mut camera_controller, mut transform)) = camera_query.single_mut() {
        let mut rotation_changed = false;
        
        for ev in mouse_motion.read() {
            camera_controller.rotation.y -= ev.delta.x * camera_controller.sensitivity;
            camera_controller.rotation.x -= ev.delta.y * camera_controller.sensitivity;
            camera_controller.rotation.x = f32::clamp(
                camera_controller.rotation.x,
                -camera_controller.rotation_lock,
                camera_controller.rotation_lock,
            );
            rotation_changed = true;
        }
        
        // Update local camera transform immediately for responsive feel
        let y_quat = Quat::from_axis_angle(Vec3::Y, camera_controller.rotation.y.to_radians());
        let x_quat = Quat::from_axis_angle(Vec3::X, camera_controller.rotation.x.to_radians());
        transform.rotation = y_quat * x_quat;
        
        // Send rotation to server if changed
        if rotation_changed {
            rotation_writer.write(RotationInput {
                yaw: camera_controller.rotation.y,
                pitch: camera_controller.rotation.x,
            });
        }
    }
}

pub mod camera_controller;
pub mod components;
pub mod input;
pub mod movement;
pub mod rendering;
pub mod shooting;
pub mod systems;

pub use rendering::{render_replicated_players, sync_remote_player_rotation, sync_player_position};
pub use systems::{spawn_players_system, handle_rotation_input, handle_movement_input, sync_transform_to_position, handle_shoot_events};

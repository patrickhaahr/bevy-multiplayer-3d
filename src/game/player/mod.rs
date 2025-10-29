pub mod camera_controller;
pub mod components;
pub mod input;
pub mod movement;
pub mod rendering;
pub mod shooting;
pub mod systems;

pub use rendering::render_replicated_players;
pub use systems::spawn_players_system;

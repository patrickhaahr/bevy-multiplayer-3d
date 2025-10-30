pub mod client;
pub mod protocol;
pub mod server;

pub use client::{client_connection_system, setup_client, ServerIpAddress};
pub use protocol::{Player, PlayerPosition, PlayerRotation, RotationInput, PORT};
pub use server::{server_connection_system, setup_server};

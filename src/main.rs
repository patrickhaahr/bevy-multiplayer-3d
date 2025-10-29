use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::RepliconRenetPlugins;

mod game;
mod network;

use game::{init_server_state, render_replicated_players, setup_world, spawn_players_system};
use network::{
    client_connection_system, server_connection_system, setup_client, setup_server, Player,
    PlayerPosition, PORT,
};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mode = if args.len() > 1 {
        args[1].as_str()
    } else {
        "server" // Default to server
    };

    match mode {
        "server" => run_server(),
        "client" => run_client(),
        _ => {
            eprintln!("Usage: {} [server|client]", args[0]);
            eprintln!("  server - Run as server (default)");
            eprintln!("  client - Run as client");
        }
    }
}

fn run_server() {
    println!("Starting headless server on port {}...", PORT);

    App::new()
        .add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            bevy::state::app::StatesPlugin,
            RepliconPlugins,
            RepliconRenetPlugins,
        ))
        .replicate::<Player>()
        .replicate::<PlayerPosition>()
        .add_systems(Startup, (setup_server, init_server_state))
        .add_systems(Update, (server_connection_system, spawn_players_system))
        .run();
}

fn run_client() {
    println!("Starting client, connecting to localhost:{}...", PORT);

    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Client".to_string(),
                    resolution: (800, 600).into(),
                    ..default()
                }),
                ..default()
            }),
            RepliconPlugins,
            RepliconRenetPlugins,
        ))
        .replicate::<Player>()
        .replicate::<PlayerPosition>()
        .add_systems(Startup, (setup_client, setup_world))
        .add_systems(Update, (client_connection_system, render_replicated_players))
        .run();
}

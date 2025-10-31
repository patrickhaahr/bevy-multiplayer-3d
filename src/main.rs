use bevy::prelude::*;
use bevy::transform::TransformPlugin;
use bevy_rapier3d::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::RepliconRenetPlugins;

mod game;
mod network;

use game::{
    cursor::CursorPlugin,
    init_server_state, render_replicated_players, sync_remote_player_rotation, sync_player_position, setup_world, setup_server_world, spawn_players_system, handle_rotation_input, handle_movement_input, sync_transform_to_position, handle_shoot_events, respawn_players_system,
    shooting::TracerPlugin,
    spawn_enemies_system, enemy_fsm_system, enemy_flocking_system, enemy_movement_system, render_enemies_system, sync_enemy_position, sync_transform_to_enemy_position,
    setup_crosshair,
};
use game::player::{
    camera_controller::update_camera_controller,
    input::PlayerInput,
    movement::{apply_local_movement, update_movement_input},
    shooting::handle_shooting,
};
use network::{
    client_connection_system, server_connection_system, setup_client, setup_server, 
    Player, PlayerPosition, PlayerRotation, Health, Enemy, EnemyPosition, PORT,
};
use network::protocol::{RotationInput, MovementInput, ShootEvent};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mode = if args.len() > 1 {
        args[1].as_str()
    } else {
        "server" // Default to server
    };

    match mode {
        "server" => run_server(),
        "client" => {
            let server_ip = if args.contains(&"--ask".to_string()) {
                // Prompt for IP address with hidden input
                use std::io::{self, Write};
                print!("Enter server IP address (default: 127.0.0.1): ");
                io::stdout().flush().unwrap();
                
                match rpassword::read_password() {
                    Ok(input) => {
                        let trimmed = input.trim();
                        if trimmed.is_empty() {
                            "127.0.0.1".to_string()
                        } else {
                            trimmed.to_string()
                        }
                    }
                    Err(_) => {
                        eprintln!("Failed to read input, using default 127.0.0.1");
                        "127.0.0.1".to_string()
                    }
                }
            } else if args.len() > 2 && args[2] != "--ask" {
                args[2].clone()
            } else {
                "127.0.0.1".to_string() // Default to localhost
            };
            run_client(server_ip)
        }
        _ => {
            eprintln!("Usage: {} [server|client] [server_ip|--ask]", args[0]);
            eprintln!("  server - Run as server (default)");
            eprintln!("  client [server_ip] - Run as client (default server_ip: 127.0.0.1)");
            eprintln!("  client --ask - Prompt for server IP address");
            eprintln!("\nExamples:");
            eprintln!("  {} client 192.168.1.100", args[0]);
            eprintln!("  {} client --ask", args[0]);
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
            TransformPlugin,
            RepliconPlugins,
            RepliconRenetPlugins,
            RapierPhysicsPlugin::<NoUserData>::default(),
        ))
        .replicate::<Player>()
        .replicate::<PlayerPosition>()
        .replicate::<PlayerRotation>()
        .replicate::<Health>()
        .replicate::<Enemy>()
        .replicate::<EnemyPosition>()
        .add_client_message::<RotationInput>(Channel::Unordered)
        .add_client_message::<MovementInput>(Channel::Unordered)
        .add_client_message::<ShootEvent>(Channel::Unordered)
        .add_systems(Startup, (setup_server, init_server_state, setup_server_world))
        .add_systems(Update, (server_connection_system, spawn_players_system, spawn_enemies_system, enemy_fsm_system, enemy_flocking_system, enemy_movement_system, handle_rotation_input, handle_movement_input, handle_shoot_events, respawn_players_system, sync_transform_to_position, sync_transform_to_enemy_position))
        .run();
}

fn run_client(server_ip: String) {
    let display_ip = if server_ip == "127.0.0.1" || server_ip == "localhost" {
        server_ip.clone()
    } else {
        "<custom>".to_string()
    };
    println!("Starting client, connecting to {}:{}...", display_ip, PORT);

    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "FPS Multiplayer Client".to_string(),
                    resolution: (1280, 720).into(),
                    ..default()
                }),
                ..default()
            }),
            RepliconPlugins,
            RepliconRenetPlugins,
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin::default(), // Physics debug visualization
            CursorPlugin,
            TracerPlugin,
        ))
        .replicate::<Player>()
        .replicate::<PlayerPosition>()
        .replicate::<PlayerRotation>()
        .replicate::<Health>()
        .replicate::<Enemy>()
        .replicate::<EnemyPosition>()
        .add_client_message::<RotationInput>(Channel::Unordered)
        .add_client_message::<MovementInput>(Channel::Unordered)
        .add_client_message::<ShootEvent>(Channel::Unordered)
        .init_resource::<PlayerInput>()
        .insert_resource(network::ServerIpAddress(server_ip))
        .add_systems(Startup, (setup_client, setup_world, setup_crosshair))
        .add_systems(
            Update,
            (
                client_connection_system,
                render_replicated_players,
                render_enemies_system,
                sync_enemy_position,
                sync_remote_player_rotation,
                sync_player_position,
                update_camera_controller,
                update_movement_input,
                apply_local_movement,
                handle_shooting,
            ),
        )
        .run();
}

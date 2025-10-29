use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::{
    renet::{ConnectionConfig, RenetClient, RenetServer, ServerEvent},
    RenetChannelsExt, RepliconRenetPlugins,
};
use renet_netcode::{
    ClientAuthentication, NetcodeClientTransport, NetcodeServerTransport,
    ServerAuthentication, ServerConfig,
};
use serde::{Deserialize, Serialize};
use std::{
    net::{Ipv4Addr, SocketAddr, UdpSocket},
    time::SystemTime,
};

const PORT: u16 = 5000;
const PROTOCOL_ID: u64 = 0;

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
        .add_systems(Startup, setup_server)
        .add_systems(Update, (
            server_connection_system,
            spawn_players_system,
        ))
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
        .add_systems(Startup, setup_client)
        .add_systems(Update, (
            client_connection_system,
            render_replicated_players,
        ))
        .run();
}

// Components
#[derive(Component, Serialize, Deserialize, Clone)]
struct Player {
    id: u64,
    color_index: u8,
}

#[derive(Component, Serialize, Deserialize)]
struct PlayerPosition {
    x: f32,
    y: f32,
    z: f32,
}

// Resource to track player count for positioning
#[derive(Resource)]
struct PlayerCount(u32);

// Marker component for rendered players on client
#[derive(Component)]
struct RenderedPlayer;

// Server setup
fn setup_server(mut commands: Commands, channels: Res<RepliconChannels>) {
    // Create server socket
    let server_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), PORT);
    let socket = UdpSocket::bind(server_addr).expect("Failed to bind server socket");
    
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    
    let server_config = ServerConfig {
        current_time,
        max_clients: 10,
        protocol_id: PROTOCOL_ID,
        public_addresses: vec![server_addr],
        authentication: ServerAuthentication::Unsecure,
    };
    
    // Get Renet channel configs
    let channel_configs = channels.server_configs();
    
    let transport = NetcodeServerTransport::new(server_config, socket)
        .expect("Failed to create server transport");
    
    let server = RenetServer::new(ConnectionConfig {
        available_bytes_per_tick: 1024 * 1024,
        client_channels_config: channel_configs.clone(),
        server_channels_config: channel_configs,
    });
    
    commands.insert_resource(server);
    commands.insert_resource(transport);
    commands.insert_resource(PlayerCount(0));
    
    println!("Server ready and listening on {}", server_addr);
}

// Client setup
fn setup_client(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    channels: Res<RepliconChannels>,
) {
    let server_addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), PORT);
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))
        .expect("Failed to bind client socket");
    
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    
    let client_id = current_time.as_millis() as u64;
    let authentication = ClientAuthentication::Unsecure {
        client_id,
        protocol_id: PROTOCOL_ID,
        server_addr,
        user_data: None,
    };
    
    let transport = NetcodeClientTransport::new(current_time, authentication, socket)
        .expect("Failed to create client transport");
    
    // Get Renet channel configs
    let channel_configs = channels.server_configs();
    
    let client = RenetClient::new(ConnectionConfig {
        available_bytes_per_tick: 1024 * 1024,
        client_channels_config: channel_configs.clone(),
        server_channels_config: channel_configs,
    });
    
    commands.insert_resource(client);
    commands.insert_resource(transport);
    
    // Spawn camera looking at the center
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 8.0, 12.0).looking_at(Vec3::new(0.0, 0.5, 0.0), Vec3::Y),
    ));
    
    // Add lighting
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, 0.5, 0.0)),
    ));
    
    // Add ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 300.0,
        affects_lightmapped_meshes: false,
    });
    
    // Ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(10.0)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.7, 0.3),
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
    
    println!("Client connecting to {}", server_addr);
}

// Server systems
fn server_connection_system(
    mut server_events: MessageReader<ServerEvent>,
) {
    for event in server_events.read() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                println!("Client {} connected", client_id);
            }
            ServerEvent::ClientDisconnected { client_id, reason } => {
                println!("Client {} disconnected: {}", client_id, reason);
            }
        }
    }
}

fn spawn_players_system(
    mut commands: Commands,
    mut server_events: MessageReader<ServerEvent>,
    mut player_count: ResMut<PlayerCount>,
) {
    for event in server_events.read() {
        if let ServerEvent::ClientConnected { client_id } = event {
            // Position players in a circle around the origin
            let angle = player_count.0 as f32 * std::f32::consts::TAU / 4.0; // Distribute evenly
            let radius = 3.0;
            let x = angle.cos() * radius;
            let z = angle.sin() * radius;
            
            let color_index = player_count.0 as u8;
            player_count.0 += 1;
            
            // Spawn player entity (server only tracks data, no rendering)
            let player_entity = commands.spawn((
                Player { id: *client_id, color_index },
                PlayerPosition { x, y: 0.5, z },
                Replicated,
            )).id();
            
            println!("Spawned player entity {:?} for client {} at position ({}, 0.5, {})", 
                     player_entity, client_id, x, z);
        }
    }
}

// Client systems
fn client_connection_system(
    client: Option<Res<RenetClient>>,
) {
    if let Some(client) = client {
        if client.is_connected() {
            // Connected
        } else if client.is_disconnected() {
            println!("Disconnected from server");
        }
    }
}

fn render_replicated_players(
    mut commands: Commands,
    players: Query<(Entity, &Player, &PlayerPosition), Without<RenderedPlayer>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, player, pos) in players.iter() {
        println!("Rendering player {} at ({}, {}, {})", player.id, pos.x, pos.y, pos.z);
        
        // Different colors for different players
        let color = match player.color_index % 5 {
            0 => Color::srgb(0.8, 0.3, 0.3), // Red
            1 => Color::srgb(0.3, 0.8, 0.3), // Green
            2 => Color::srgb(0.3, 0.3, 0.8), // Blue
            3 => Color::srgb(0.8, 0.8, 0.3), // Yellow
            _ => Color::srgb(0.8, 0.3, 0.8), // Magenta
        };
        
        // Add rendering components to the replicated entity
        commands.entity(entity).insert((
            RenderedPlayer,
            Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                ..default()
            })),
            Transform::from_xyz(pos.x, pos.y, pos.z),
        ));
    }
}

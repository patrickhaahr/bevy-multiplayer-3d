use bevy::prelude::*;
use bevy_replicon::prelude::RepliconChannels;
use bevy_replicon_renet::{
    renet::{ConnectionConfig, RenetClient},
    RenetChannelsExt,
};
use renet_netcode::{ClientAuthentication, NetcodeClientTransport};
use std::{
    net::{SocketAddr, UdpSocket},
    time::SystemTime,
};

use super::protocol::PORT;
use super::protocol::PROTOCOL_ID;

// Resource to track the local client ID
#[derive(Resource)]
pub struct LocalClientId(pub u64);

// Resource to store the server IP address
#[derive(Resource)]
pub struct ServerIpAddress(pub String);

pub fn setup_client(
    mut commands: Commands,
    channels: Res<RepliconChannels>,
    server_ip: Res<ServerIpAddress>,
) {
    let server_addr: SocketAddr = format!("{}:{}", server_ip.0, PORT)
        .parse()
        .expect("Invalid server IP address");
    
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind client socket");

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
    let server_channels = channels.server_configs();
    let client_channels = channels.client_configs();

    let client = RenetClient::new(ConnectionConfig {
        available_bytes_per_tick: 1024 * 1024,
        server_channels_config: server_channels,
        client_channels_config: client_channels,
    });

    commands.insert_resource(client);
    commands.insert_resource(transport);
    commands.insert_resource(LocalClientId(client_id));

    let display_addr = if server_ip.0 == "127.0.0.1" || server_ip.0 == "localhost" {
        server_addr.to_string()
    } else {
        format!("<custom>:{}", PORT)
    };
    println!("Client connecting to {} with ID {}", display_addr, client_id);
}

pub fn client_connection_system(client: Option<Res<RenetClient>>) {
    if let Some(client) = client {
        if client.is_connected() {
            // Connected
        } else if client.is_disconnected() {
            println!("Disconnected from server");
        }
    }
}

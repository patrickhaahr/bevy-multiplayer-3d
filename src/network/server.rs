use bevy::prelude::*;
use bevy_replicon::prelude::RepliconChannels;
use bevy_replicon_renet::{
    renet::{ConnectionConfig, RenetServer, ServerEvent},
    RenetChannelsExt,
};
use renet_netcode::{NetcodeServerTransport, ServerAuthentication, ServerConfig};
use std::{
    net::{Ipv4Addr, SocketAddr, UdpSocket},
    time::SystemTime,
};

use super::protocol::PORT;
use super::protocol::PROTOCOL_ID;

pub fn setup_server(mut commands: Commands, channels: Res<RepliconChannels>) {
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

    println!("Server ready and listening on {}", server_addr);
}

pub fn server_connection_system(mut server_events: EventReader<ServerEvent>) {
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

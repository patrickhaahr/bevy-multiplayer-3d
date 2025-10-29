use bevy::prelude::*;
use bevy_replicon::prelude::RepliconChannels;
use bevy_replicon_renet::{
    renet::{ConnectionConfig, RenetClient},
    RenetChannelsExt,
};
use renet_netcode::{ClientAuthentication, NetcodeClientTransport};
use std::{
    net::{Ipv4Addr, SocketAddr, UdpSocket},
    time::SystemTime,
};

use super::protocol::PORT;
use super::protocol::PROTOCOL_ID;

pub fn setup_client(mut commands: Commands, channels: Res<RepliconChannels>) {
    let server_addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), PORT);
    let socket =
        UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).expect("Failed to bind client socket");

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

    println!("Client connecting to {}", server_addr);
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

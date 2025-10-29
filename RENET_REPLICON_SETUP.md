# Bevy Replicon + Renet Setup Guide

This guide shows key setup patterns for multiplayer games using:
- **bevy** = "0.17.2"
- **bevy_replicon** = "0.36.0"
- **bevy_replicon_renet** = "0.12"
- **renet** = "1.2.0"

## Core Architecture Pattern

### Server-Client Model
- Server owns authoritative game state (position, health, inventory, etc.)
- Server replicates state to clients each tick
- Clients send input events only (not state)
- Server validates & processes input, updates state

---

## 1. Plugin Setup

```rust
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::RepliconRenetPlugins;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            RepliconPlugins,      // Core replication plugin
            RepliconRenetPlugins, // Renet messaging backend
        ))
        // Register replicated components
        .replicate::<YourComponent>()
        // Register client events
        .add_client_event::<YourEvent>(Channel::Ordered)
        .run();
}
```

**Important**: `RepliconRenetPlugins` automatically includes Renet plugins, so don't add them separately.

---

## 2. Channel Setup

Channels must be configured to match Replicon's channel system:

```rust
use bevy_replicon::prelude::*;
use bevy_replicon_renet::{
    renet::{ConnectionConfig, RenetClient, RenetServer},
    RenetChannelsExt,
};

fn setup_server(channels: Res<RepliconChannels>) {
    let connection_config = ConnectionConfig {
        server_channels_config: channels.server_configs(),
        client_channels_config: channels.client_configs(),
        ..Default::default()
    };
    
    let server = RenetServer::new(connection_config);
    // Insert server resource...
}
```

**Critical**: Only get channels AFTER registering all replicated components and events!

---

## 3. Server Setup

```rust
use bevy_replicon_renet::{
    renet::{ConnectionConfig, RenetServer},
    netcode::{NetcodeServerTransport, ServerAuthentication, ServerConfig},
    RenetChannelsExt,
};
use std::{net::{Ipv4Addr, UdpSocket}, time::SystemTime};

fn setup_server(
    mut commands: Commands,
    channels: Res<RepliconChannels>,
) -> Result<()> {
    const PROTOCOL_ID: u64 = 0; // Must match client
    const PORT: u16 = 5000;
    
    // Create Renet server
    let server = RenetServer::new(ConnectionConfig {
        server_channels_config: channels.server_configs(),
        client_channels_config: channels.client_configs(),
        ..Default::default()
    });
    
    // Setup transport
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?;
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, PORT))?;
    let server_config = ServerConfig {
        current_time,
        max_clients: 10,
        protocol_id: PROTOCOL_ID,
        authentication: ServerAuthentication::Unsecure,
        public_addresses: Default::default(),
    };
    let transport = NetcodeServerTransport::new(server_config, socket)?;
    
    // Insert resources
    commands.insert_resource(server);
    commands.insert_resource(transport);
    
    Ok(())
}
```

---

## 4. Client Setup

```rust
use bevy_replicon_renet::{
    renet::{ConnectionConfig, RenetClient},
    netcode::{ClientAuthentication, NetcodeClientTransport},
    RenetChannelsExt,
};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    time::SystemTime,
};

fn setup_client(
    mut commands: Commands,
    channels: Res<RepliconChannels>,
) -> Result<()> {
    const PROTOCOL_ID: u64 = 0; // Must match server
    const SERVER_IP: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);
    const SERVER_PORT: u16 = 5000;
    
    // Create Renet client
    let client = RenetClient::new(ConnectionConfig {
        server_channels_config: channels.server_configs(),
        client_channels_config: channels.client_configs(),
        ..Default::default()
    });
    
    // Setup transport
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?;
    let client_id = current_time.as_millis() as u64;
    let server_addr = SocketAddr::new(SERVER_IP, SERVER_PORT);
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))?;
    
    let authentication = ClientAuthentication::Unsecure {
        client_id,
        protocol_id: PROTOCOL_ID,
        server_addr,
        user_data: None,
    };
    let transport = NetcodeClientTransport::new(
        current_time,
        authentication,
        socket,
    )?;
    
    // Insert resources
    commands.insert_resource(client);
    commands.insert_resource(transport);
    
    Ok(())
}
```

---

## 5. Component Replication

### Basic Replication
```rust
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};

// Components MUST implement Serialize and Deserialize
#[derive(Component, Serialize, Deserialize)]
struct Health(f32);

#[derive(Component, Serialize, Deserialize)]
struct Position(Vec3);

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, RepliconPlugins, RepliconRenetPlugins))
        .replicate::<Health>()    // Replicate continuously
        .replicate::<Position>()
        .run();
}
```

### One-Time Replication
For data that doesn't change after spawn:

```rust
#[derive(Component, Serialize, Deserialize)]
struct PlayerId(u64);

fn main() {
    App::new()
        .replicate_once::<PlayerId>()  // Only replicate on spawn
        .run();
}
```

### Custom Replication Mapping
Replicate a subset of component data:

```rust
#[derive(Serialize, Deserialize)]
struct Transform2D {
    translation: Vec2,
    rotation: f32,
}

fn main() {
    App::new()
        // Replicate Transform as Transform2D
        .replicate_once_as::<Transform, Transform2D>()
        .run();
}
```

---

## 6. Client Events (Input)

Clients send events to server for processing:

```rust
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};

// Client sends this to server
#[derive(Event, Serialize, Deserialize)]
struct MoveCommand {
    direction: Vec2,
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, RepliconPlugins, RepliconRenetPlugins))
        .add_client_event::<MoveCommand>(Channel::Ordered)
        .add_observer(process_move_command)
        .run();
}

// Server processes client input
fn process_move_command(
    trigger: On<FromClient<MoveCommand>>,
    mut players: Query<&mut Position>,
) {
    let client_id = trigger.client_id;
    let event = &trigger.event;
    
    // Find player entity for this client
    // Update position based on validated input
}
```

### Mapped Entity Events
For events that reference entities:

```rust
use bevy::ecs::entity::MapEntities;

#[derive(Event, Serialize, Deserialize, MapEntities)]
struct UseItem {
    #[entities]  // This entity will be mapped between client/server
    item_entity: Entity,
}

fn main() {
    App::new()
        .add_mapped_client_event::<UseItem>(Channel::Ordered)
        .run();
}
```

---

## 7. Connection States

Replicon provides states to track connection status:

```rust
fn main() {
    App::new()
        .add_plugins((DefaultPlugins, RepliconPlugins, RepliconRenetPlugins))
        // React to server state changes
        .add_systems(OnEnter(ServerState::Running), on_server_running)
        .add_systems(OnExit(ServerState::Running), on_server_stopped)
        // React to client state changes
        .add_systems(OnEnter(ClientState::Connected), on_client_connected)
        .add_systems(OnEnter(ClientState::Connecting), on_client_connecting)
        .add_systems(OnExit(ClientState::Connected), on_client_disconnected)
        .run();
}

fn on_server_running() {
    info!("Server is now running!");
}

fn on_client_connected() {
    info!("Client connected to server!");
}
```

**Note**: States are updated in `PreUpdate`, so they won't be available in `Startup`.

---

## 8. Client Identification

Mark which player belongs to the local client:

```rust
#[derive(Component)]
struct LocalPlayer;

// On client
fn spawn_local_player(mut commands: Commands) {
    commands.spawn((
        LocalPlayer,
        Position(Vec3::ZERO),
    ));
}

// In systems, filter for local player
fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    local_player: Query<&Position, With<LocalPlayer>>,
) {
    // Only process input for local player
}
```

---

## 9. Replication Relationships

Replicate parent-child relationships:

```rust
use bevy::ecs::relationship::ChildOf;

fn main() {
    App::new()
        .replicate::<Parent>()
        // Replicate ChildOf relationship only for specific components
        .replicate_filtered::<ChildOf, With<YourComponent>>()
        .run();
}
```

---

## Key Takeaways

1. **Never insert both client and server resources in the same app** - causes replication loops
2. **Get channels only after registering components/events** - or they won't be included
3. **Protocol ID must match** between client and server
4. **Server is authoritative** - clients send input, server updates state
5. **Components must be Serialize + Deserialize** for replication
6. **Use states** (`ServerState`, `ClientState`) to react to connection changes
7. **Mark local entities** with a component like `LocalPlayer` for client-side filtering

---

## Additional Resources

- [Bevy Replicon Docs](https://docs.rs/bevy_replicon)
- [Bevy Replicon Renet Docs](https://docs.rs/bevy_replicon_renet)
- [Official Examples](https://github.com/simgine/bevy_replicon_renet/tree/master/examples)
- [Renet Repository](https://github.com/lucaspoffo/renet)

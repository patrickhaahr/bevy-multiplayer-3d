# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A multiplayer 3D FPS game built with Bevy 0.17.2, using bevy_replicon for network replication and Rapier for physics. The game follows a server-authoritative model where clients send input and the server owns all game state.

## Commands

### Build and Run
```bash
# Build the project
cargo build

# Run server (default)
cargo run

# Run server explicitly
cargo run server

# Run client
cargo run client

# Generate local documentation
cargo doc --no-deps --open
```

### Development
```bash
# Build with optimizations
cargo build --release

# Run with physics debug rendering (edit src/main.rs:83 to uncomment RapierDebugRenderPlugin)
cargo run client
```

## Technology Stack

- **Engine**: Bevy 0.17.2 with dynamic linking
- **Networking**: bevy_replicon 0.36.0 + bevy_replicon_renet 0.12
- **Transport**: renet 1.2
- **Physics**: bevy_rapier3d (custom fork)
- **Serialization**: serde 1.0

## Architecture

### Networking Model

The game uses a server-client architecture with the following principles:
- **Server Authority**: Server owns all game state (position, rotation, health, etc.)
- **State Replication**: Server replicates state to clients each tick
- **Input-Only Clients**: Clients send input events/messages only, never state
- **Server Validation**: All input is validated and processed server-side

### Network Stack Layers
```
Game Logic (src/game/)
    ↓
bevy_replicon (replication, ECS state sync)
    ↓
bevy_replicon_renet (messaging backend)
    ↓
renet (UDP transport)
    ↓
Network
```

### Module Structure

**`src/main.rs`**
- CLI argument parsing (server/client mode)
- App construction with mode-specific plugins
- System registration for server and client

**`src/network/`** - Transport and protocol layer
- `protocol.rs`: Shared network protocol (components, messages, constants)
  - Network constants: `PORT`, `PROTOCOL_ID`
  - Replicated components: `Player`, `PlayerPosition`, `PlayerRotation`
  - Client messages: `RotationInput`
  - Must be identical on server and client
- `server.rs`: Server setup (`setup_server()`, `server_connection_system()`)
- `client.rs`: Client setup (`setup_client()`, `client_connection_system()`)

**`src/game/`** - Game logic layer
- `player/`: Player entity management
  - `components.rs`: Component definitions (re-exports from protocol + client-only markers)
  - `systems.rs`: Server-side logic (`spawn_players_system()`, `handle_rotation_input()`)
  - `rendering.rs`: Client-side rendering (`render_replicated_players()`, `sync_remote_player_rotation()`)
  - `input.rs`: Input resource (`PlayerInput`)
  - `movement.rs`: Movement systems (`update_movement_input()`, `apply_local_movement()`)
  - `shooting.rs`: Shooting logic (`handle_shooting()`)
  - `camera_controller.rs`: First-person camera (`update_camera_controller()`)
- `world/`: World state and environment
  - `state.rs`: Server resources (e.g., `PlayerCount`)
  - `setup.rs`: Environment setup (`setup_world()`, `init_server_state()`)
- `cursor/`: Cursor lock and visibility
- `shooting/`: Bullet tracers and effects

### Replication Setup

Components are registered for replication in `main.rs`:
```rust
.replicate::<Player>()
.replicate::<PlayerPosition>()
.replicate::<PlayerRotation>()
```

Client messages (input) are registered:
```rust
.add_client_message::<RotationInput>(Channel::Unordered)
```

### Server vs Client Systems

**Server-only systems** (run in `run_server()`):
- Connection handling
- Player spawning
- Input processing and validation
- State updates

**Client-only systems** (run in `run_client()`):
- Rendering (models, UI)
- Camera control
- Input collection
- Visual effects (tracers, particles)

## Critical Development Guidelines

### Documentation Usage

**ALWAYS consult local API documentation before making changes**. Online docs are often outdated due to rapid API evolution in Bevy and networking crates.

**Local docs location**: `target/doc/`
- Bevy core: `target/doc/bevy/`
- Networking:
  - `target/doc/renet/` - UDP transport
  - `target/doc/bevy_replicon/` - Replication framework
  - `target/doc/bevy_replicon_renet/` - Integration layer

**Source code inspection**: If docs are unclear, check `~/.cargo/registry/src/` for actual implementation.

### Adding New Features

**Adding replicated components:**
1. Define in `src/network/protocol.rs` with `#[derive(Component, Serialize, Deserialize)]`
2. Add `.replicate::<YourComponent>()` in both server and client app setup in `main.rs`
3. Use in game systems

**Adding client messages (input):**
1. Define in `src/network/protocol.rs` with `#[derive(Message, Serialize, Deserialize)]`
2. Add `.add_client_message::<YourMessage>(Channel::Unordered)` in both apps
3. Handle on server side

**Adding server logic:**
- Add systems to appropriate module in `src/game/player/` or create new module
- Register in `main.rs` under `run_server()`

**Adding client rendering:**
- Add systems to `src/game/player/rendering.rs` or appropriate module
- Register in `main.rs` under `run_client()`

### File Organization Rules

When creating new files:
- Network components/messages/constants → `src/network/protocol.rs`
- Server networking → `src/network/server.rs`
- Client networking → `src/network/client.rs`
- Player components → `src/game/player/components.rs`
- Player server logic → `src/game/player/systems.rs`
- Player rendering → `src/game/player/rendering.rs`
- Player input handling → `src/game/player/input.rs` or `movement.rs`
- World resources → `src/game/world/state.rs`
- World setup → `src/game/world/setup.rs`
- 3D assets → `assets/models/`

### Common Pitfalls

1. **Never insert both RenetClient and RenetServer in the same app** - causes replication loops
2. **Get RepliconChannels only AFTER registering all components/events** - or channels won't include them
3. **Protocol ID must match** between client and server
4. **Server is authoritative** - validate all client input, never trust client state
5. **Use Channel::Ordered for critical messages, Channel::Unordered for frequent non-critical updates**

## Additional Documentation

See `AGENTS.md` for detailed architecture and module responsibilities.
See `RENET_REPLICON_SETUP.md` for networking setup patterns and examples.

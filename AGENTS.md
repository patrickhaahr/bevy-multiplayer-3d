# Multiplayer Game Architecture

## Server-Client Model
- Server owns game state (position, health, inventory)
- Server replicates state → clients each tick
- Clients send input events only (not state)
- Server validates & processes input, updates state

## Implementation Details

### Technology Stack
- **Engine**: Bevy 0.17.2 with dynamic linking
- **Networking**: bevy_replicon 0.36.0 + bevy_replicon_renet 0.12
- **Transport**: renet 1.2
- **Serialization**: serde 1.0

## Documentation

**CRITICAL**: Always consult local API documentation before making changes. Bevy and its networking plugins have undergone significant API changes, and online docs may be outdated.

### Local Documentation Paths
All documentation is generated locally in `target/doc/`:
- **Bevy core**: `target/doc/bevy/`
- **Networking**:
  - `target/doc/renet/` - UDP transport layer
  - `target/doc/bevy_replicon/` - Replication framework
  - `target/doc/bevy_replicon_renet/` - Replicon-Renet integration

### Source Code Access
If documentation is insufficient, you can inspect the actual source code of dependencies:
- **Crate sources**: Located in `~/.cargo/registry/src/`
- **Bevy_rapier3d**: `~/.cargo/git/checkouts/bevy_rapier-0ed6fb17c9a8da50/a5c82f0/` - Rapier physics engine. we are using a custom commit of this plugin.
- Use this when you need to understand implementation details, see examples in the source, or verify behavior

### Workflow
1. **Before implementing**: Check `target/doc/` for the relevant crate's API
2. **When in doubt**: Verify function signatures, traits, and types in local docs
3. **If docs are unclear**: Inspect the source code directly in `~/.cargo/registry/src/`
4. **API changes**: Assume online documentation may be outdated; trust local docs as the source of truth
5. **APIs are rapidly evolving**: Bevy and networking crates frequently introduce breaking changes between versions - always verify against local docs

## Network Architecture

The game uses a layered networking stack:

```
Your Game Logic
    ↓
bevy_replicon (replication, ECS state sync)
    ↓
bevy_replicon_renet (messaging backend integration)
    ↓
renet (UDP transport)
    ↓
Network
```

## Code Architecture

### Directory Structure
```
mp/
├── assets/
│   └── models/           # 3D models (player.glb, gun.glb)
├── src/
│   ├── main.rs          # Entry point, app setup
│   ├── network/         # Network layer
│   │   ├── mod.rs
│   │   ├── protocol.rs  # Shared protocol (components, constants)
│   │   ├── server.rs    # Server setup & connection systems
│   │   └── client.rs    # Client setup & connection systems
│   └── game/            # Game logic layer
│       ├── mod.rs
│       ├── player/      # Player subsystem
│       │   ├── mod.rs
│       │   ├── components.rs  # Player components
│       │   ├── systems.rs     # Server-side player logic
│       │   └── rendering.rs   # Client-side rendering
│       └── world/       # World subsystem
│           ├── mod.rs
│           ├── state.rs       # World state resources
│           └── setup.rs       # World setup (camera, lighting, ground)
```

### Module Responsibilities

#### **`src/main.rs`**
- CLI argument parsing (server/client mode)
- App construction with appropriate plugins
- System registration
- Minimal orchestration logic

#### **`src/network/`**
Network layer handling transport and protocol:

- **`protocol.rs`**: Shared definitions
  - Network constants (`PORT`, `PROTOCOL_ID`)
  - Replicated components (`Player`, `PlayerPosition`)
  - Must be identical on server and client

- **`server.rs`**: Server networking
  - `setup_server()` - Initialize RenetServer and transport
  - `server_connection_system()` - Handle client connect/disconnect events
  
- **`client.rs`**: Client networking
  - `setup_client()` - Initialize RenetClient and transport
  - `client_connection_system()` - Monitor connection status

#### **`src/game/player/`**
Player entity management:

- **`components.rs`**: Component definitions
  - Re-exports `Player` and `PlayerPosition` from protocol
  - `RenderedPlayer` marker (client-only)

- **`systems.rs`**: Server-side logic
  - `spawn_players_system()` - Creates player entities when clients connect
  - Future: input handling, movement validation, combat

- **`rendering.rs`**: Client-side rendering
  - `render_replicated_players()` - Loads and attaches 3D models to replicated entities
  - Uses `assets/models/player.glb` and `gun.glb`

#### **`src/game/world/`**
World state and environment:

- **`state.rs`**: Resources
  - `PlayerCount` - Tracks connected players (server-side)
  - Future: game phase, tick counter, world parameters

- **`setup.rs`**: Environment setup
  - `setup_world()` - Camera, lighting, ground plane (client-side)
  - `init_server_state()` - Initialize server resources

### Design Principles

1. **Separation of Concerns**
   - Network layer is transport-agnostic
   - Game logic doesn't know about UDP/sockets
   - Clear server/client separation

2. **Server Authority**
   - Server owns all game state
   - Clients only render and send input
   - All validation happens server-side

3. **Component-Based Architecture**
   - Players are entities with components
   - Systems operate on queries
   - Easy to extend with new components/systems

### Adding New Features

**Adding a new replicated component:**
1. Define in `src/network/protocol.rs` with `#[derive(Component, Serialize, Deserialize)]`
2. Add `.replicate::<YourComponent>()` in `main.rs` for both server and client
3. Use in game systems

**Adding server-side logic:**
- Add systems to `src/game/player/systems.rs` or create new modules
- Register in `main.rs` under `run_server()`

**Adding client-side rendering:**
- Add rendering logic to `src/game/player/rendering.rs`
- Register in `main.rs` under `run_client()`

**Adding world features:**
- State resources go in `src/game/world/state.rs`
- Setup logic goes in `src/game/world/setup.rs`

### File Location Guidelines

When creating new files, follow these rules:

- **Network components/constants** → `src/network/protocol.rs`
- **Server networking code** → `src/network/server.rs`
- **Client networking code** → `src/network/client.rs`
- **Player components** → `src/game/player/components.rs`
- **Player server logic** → `src/game/player/systems.rs`
- **Player rendering** → `src/game/player/rendering.rs`
- **World resources** → `src/game/world/state.rs`
- **World setup** → `src/game/world/setup.rs`
- **3D models** → `assets/models/`

Keep the architecture clean by respecting module boundaries and avoiding circular dependencies.

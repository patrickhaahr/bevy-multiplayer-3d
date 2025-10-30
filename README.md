# Multiplayer 3D Shooter

A server-authoritative multiplayer 3D shooter built with Rust and the Bevy game engine. Players can connect to a dedicated server, move around a 3D world, and engage in combat.

## What is This?

This is a networked multiplayer game where:
- **Server** owns and validates all game state (player positions, health, inventory)
- **Clients** send input commands and render the replicated game state
- **Server-authoritative model** prevents cheating and ensures consistent gameplay

The game features:
- Third-person 3D player movement
- Shooting mechanics with visual tracers
- Real-time state replication between server and clients
- Physics-based interactions

## Technology Stack

### Core Engine
- **[Bevy 0.17.2](https://bevyengine.org/)** - Data-driven game engine built in Rust using ECS (Entity Component System) architecture

### Networking
- **[bevy_replicon 0.36.0](https://docs.rs/bevy_replicon/)** - High-level replication framework for Bevy that automatically synchronizes entities and components between server and clients
- **[bevy_replicon_renet 0.12](https://docs.rs/bevy_replicon_renet/)** - Integration layer connecting bevy_replicon with the renet transport
- **[renet 1.2](https://docs.rs/renet/)** - UDP-based networking library providing reliable and unreliable channels

### Physics
- **[bevy_rapier3d](https://github.com/dimforge/bevy_rapier)** - 3D physics engine integration for Bevy, handling collisions, rigid bodies, and physical interactions

### Assets
All 3D models are created in **Blender** and exported as `.glb` files:
- `assets/models/player.glb` - Player character model
- `assets/models/gun.glb` - Weapon model
- `assets/models/environment.glb` - Environment decoration

## How to Play

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (latest stable version)
- Bevy engine dependencies for your platform:
  - **Linux**: `sudo apt-get install g++ pkg-config libx11-dev libasound2-dev libudev-dev libxkbcommon-x11-0`
  - **Fedora**: `sudo dnf install gcc-c++ libX11-devel alsa-lib-devel systemd-devel`
  - **Arch**: `sudo pacman -S libx11 pkgconf alsa-lib`
  - **macOS**: Xcode command line tools
  - **Windows**: No additional dependencies required
- A terminal/command prompt

### Starting a Server

```bash
cargo run -- server
```

The server will start listening on `0.0.0.0:5000` (all network interfaces).

### Starting a Client

**Connect to localhost:**
```bash
cargo run -- client
```

**Connect to a specific IP address:**
```bash
cargo run -- client <SERVER_IP>
```

For example, to connect to a server at `192.168.1.100`:
```bash
cargo run -- client 192.168.1.100
```

### Controls
- **WASD** - Move player
- **Mouse** - Look around
- **Left Click** - Shoot
- **ESC** - Release mouse cursor

## Development

### Building for Release
```bash
cargo build --release
```

Release binaries will be in `target/release/mp`.

### Running Server and Client Separately
```bash
# Terminal 1 - Server
./target/release/mp server

# Terminal 2 - Client
./target/release/mp client
```

### Network Configuration
- **Port**: 5000 (UDP)
- **Protocol ID**: `MULTIPLAYER_SHOOTER`
- Default server IP: `127.0.0.1` (localhost)

## Architecture

```
Client Input → Server Validation → State Update → Replicate to All Clients
```

The server runs at a fixed tick rate, processes player inputs, updates physics, and replicates the authoritative game state back to all connected clients. Clients render this state and send their input commands to the server.

For more details, see `AGENTS.md`.

## License

This project is for educational purposes.

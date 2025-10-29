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


# Asset Placeholders

This directory contains 3D models for the game.

## Required Models

Place the following GLB files in this directory:

- **player.glb** - Main player character model
- **gun.glb** - Gun/weapon model (attached to player)

## Loading

Models are loaded using Bevy's `AssetServer` in `src/game/player/rendering.rs`:
- Player model: `assets/models/player.glb#Scene0`
- Gun model: `assets/models/gun.glb#Scene0`

The gun is spawned as a child entity of the player, positioned relative to the player's transform.

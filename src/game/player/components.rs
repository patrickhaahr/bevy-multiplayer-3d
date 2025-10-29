use bevy::prelude::*;

// Marker component for rendered players on client
#[derive(Component)]
pub struct RenderedPlayer;

// Marker for the player model (body visible to others)
#[derive(Component)]
pub struct PlayerModel;

// Marker for gun model
#[derive(Component)]
pub struct GunModel;

// Marker for the tracer spawn point
#[derive(Component)]
pub struct TracerSpawnSpot;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

// Marker component for rendered players on client
#[derive(Component)]
pub struct RenderedPlayer;

// Marker for gun model
#[derive(Component)]
pub struct GunModel;

// Marker for the tracer spawn point
#[derive(Component)]
pub struct TracerSpawnSpot;

// Bundle for player physics components
#[derive(Bundle)]
pub struct PlayerPhysicsBundle {
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub locked_axes: LockedAxes,
    pub velocity: Velocity,
    pub mass: AdditionalMassProperties,
    pub gravity_scale: GravityScale,
    pub damping: Damping,
    pub friction: Friction,
    pub restitution: Restitution,
}

impl Default for PlayerPhysicsBundle {
    fn default() -> Self {
        Self {
            rigid_body: RigidBody::Dynamic,
            // Capsule: half-height=0.9, radius=0.3 (total height ~2.1 units)
            collider: Collider::capsule_y(0.9, 0.3),
            // Lock all rotations so player stays upright
            locked_axes: LockedAxes::ROTATION_LOCKED,
            velocity: Velocity::default(),
            // Add mass so physics works properly
            mass: AdditionalMassProperties::Mass(80.0), // 80kg player
            gravity_scale: GravityScale(1.0), // Full gravity
            damping: Damping {
                linear_damping: 0.5, // Slow down when not moving
                angular_damping: 1.0,
            },
            friction: Friction {
                coefficient: 0.7,
                combine_rule: CoefficientCombineRule::Average,
            },
            restitution: Restitution {
                coefficient: 0.0, // No bounciness
                combine_rule: CoefficientCombineRule::Min,
            },
        }
    }
}

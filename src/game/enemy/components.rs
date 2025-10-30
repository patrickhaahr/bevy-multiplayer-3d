use bevy::prelude::*;

/// FSM states for enemy (server-only, not replicated)
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub enum EnemyState {
    Patrol,
    Chase,
    Attack,
}

/// Patrol waypoint index (server-only, pure data component)
#[derive(Component)]
pub struct PatrolData {
    pub waypoints: Vec<Vec3>,
    pub current_waypoint: usize,
}

impl PatrolData {
    pub fn new(center: Vec3, radius: f32) -> Self {
        // Create a simple square patrol pattern
        let waypoints = vec![
            center + Vec3::new(radius, 0.0, radius),
            center + Vec3::new(radius, 0.0, -radius),
            center + Vec3::new(-radius, 0.0, -radius),
            center + Vec3::new(-radius, 0.0, radius),
        ];

        Self {
            waypoints,
            current_waypoint: 0,
        }
    }
}

// Helper functions for patrol logic (to be used in systems)
pub fn get_current_waypoint(patrol: &PatrolData) -> Vec3 {
    patrol.waypoints[patrol.current_waypoint]
}

pub fn advance_waypoint(patrol: &mut PatrolData) {
    patrol.current_waypoint = (patrol.current_waypoint + 1) % patrol.waypoints.len();
}

/// Enemy movement parameters (server-only)
#[derive(Component)]
pub struct EnemyMovement {
    pub chase_range: f32,
    pub attack_range: f32,
    pub patrol_speed: f32,
    pub chase_speed: f32,
}

impl Default for EnemyMovement {
    fn default() -> Self {
        Self {
            chase_range: 4.0,
            attack_range: 2.5,
            patrol_speed: 2.0,
            chase_speed: 4.0, // Slower than player
        }
    }
}

/// Flocking behavior parameters (server-only)
#[derive(Component)]
pub struct FlockingBehavior {
    /// Range within which enemies are considered neighbors
    pub neighbor_range: f32,
    /// Weight for cohesion (move towards center of flock)
    pub cohesion_weight: f32,
    /// Weight for alignment (match velocity with neighbors)
    pub alignment_weight: f32,
    /// Weight for separation (avoid crowding)
    pub separation_weight: f32,
    /// Minimum distance to maintain from neighbors
    pub separation_distance: f32,
}

impl Default for FlockingBehavior {
    fn default() -> Self {
        Self {
            neighbor_range: 10.0,
            cohesion_weight: 0.8,
            alignment_weight: 0.8,
            separation_weight: 3.0, // Much stronger separation to prevent collision
            separation_distance: 2.5, // Larger minimum distance (capsule radius is 0.5, so 2.5 gives good spacing)
        }
    }
}

/// Velocity component for enemies (server-only)
#[derive(Component, Default)]
pub struct EnemyVelocity {
    pub velocity: Vec3,
}

/// Client-side marker for rendered enemies
#[derive(Component)]
pub struct RenderedEnemy;

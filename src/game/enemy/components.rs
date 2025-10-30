use bevy::prelude::*;

/// FSM states for enemy (server-only, not replicated)
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub enum EnemyState {
    Patrol,
    Chase,
    Attack,
}

/// Patrol waypoint index (server-only)
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

    pub fn get_current_waypoint(&self) -> Vec3 {
        self.waypoints[self.current_waypoint]
    }

    pub fn advance(&mut self) {
        self.current_waypoint = (self.current_waypoint + 1) % self.waypoints.len();
    }
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

/// Client-side marker for rendered enemies
#[derive(Component)]
pub struct RenderedEnemy;

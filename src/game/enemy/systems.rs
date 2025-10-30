use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_replicon::prelude::*;
use crate::network::protocol::{PlayerPosition, Enemy, EnemyPosition};
use super::components::{EnemyState, PatrolData, EnemyMovement, RenderedEnemy, FlockingBehavior, EnemyVelocity, get_current_waypoint, advance_waypoint};

/// Spawn enemies in the world (server-side)
pub fn spawn_enemies_system(
    mut commands: Commands,
    enemies: Query<&Enemy>,
) {
    // Spawn 3 enemies in a cluster
    if enemies.iter().count() > 0 {
        return;
    }
    
    // Spawn 3 enemies in a triangle formation to demonstrate flocking
    let cluster_center = Vec3::new(10.0, 1.0, 10.0);
    let spawn_radius = 3.0;
    
    // Enemy 1 - north
    let spawn_pos_1 = cluster_center + Vec3::new(0.0, 0.0, spawn_radius);
    commands.spawn((
        Enemy { id: 1 },
        EnemyPosition {
            x: spawn_pos_1.x,
            y: spawn_pos_1.y,
            z: spawn_pos_1.z,
        },
        EnemyState::Patrol,
        PatrolData::new(cluster_center, 5.0),
        EnemyMovement::default(),
        FlockingBehavior::default(),
        EnemyVelocity::default(),
        Transform::from_translation(spawn_pos_1),
        GlobalTransform::default(),
        Collider::capsule_y(0.5, 0.5),
        RigidBody::KinematicPositionBased,
        Replicated,
    ));
    info!("Spawned enemy 1 at {:?}", spawn_pos_1);
    
    // Enemy 2 - southwest
    let spawn_pos_2 = cluster_center + Vec3::new(-spawn_radius * 0.866, 0.0, -spawn_radius * 0.5);
    commands.spawn((
        Enemy { id: 2 },
        EnemyPosition {
            x: spawn_pos_2.x,
            y: spawn_pos_2.y,
            z: spawn_pos_2.z,
        },
        EnemyState::Patrol,
        PatrolData::new(cluster_center, 5.0),
        EnemyMovement::default(),
        FlockingBehavior::default(),
        EnemyVelocity::default(),
        Transform::from_translation(spawn_pos_2),
        GlobalTransform::default(),
        Collider::capsule_y(0.5, 0.5),
        RigidBody::KinematicPositionBased,
        Replicated,
    ));
    info!("Spawned enemy 2 at {:?}", spawn_pos_2);
    
    // Enemy 3 - southeast
    let spawn_pos_3 = cluster_center + Vec3::new(spawn_radius * 0.866, 0.0, -spawn_radius * 0.5);
    commands.spawn((
        Enemy { id: 3 },
        EnemyPosition {
            x: spawn_pos_3.x,
            y: spawn_pos_3.y,
            z: spawn_pos_3.z,
        },
        EnemyState::Patrol,
        PatrolData::new(cluster_center, 5.0),
        EnemyMovement::default(),
        FlockingBehavior::default(),
        EnemyVelocity::default(),
        Transform::from_translation(spawn_pos_3),
        GlobalTransform::default(),
        Collider::capsule_y(0.5, 0.5),
        RigidBody::KinematicPositionBased,
        Replicated,
    ));
    info!("Spawned enemy 3 at {:?}", spawn_pos_3);
}

/// Client-side rendering for enemies (add visual mesh)
pub fn render_enemies_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    enemies: Query<Entity, (With<Enemy>, Without<RenderedEnemy>)>,
) {
    for entity in enemies.iter() {
        commands.entity(entity).insert((
            RenderedEnemy,
            Mesh3d(meshes.add(Capsule3d::new(0.5, 1.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.0, 0.0),
                ..default()
            })),
            Visibility::default(),
            InheritedVisibility::default(),
            ViewVisibility::default(),
        ));
        
        info!("Added rendering to enemy {:?}", entity);
    }
}

/// Sync replicated position to Transform on client
pub fn sync_enemy_position(
    mut enemies: Query<(&EnemyPosition, &mut Transform), (With<Enemy>, Without<PlayerPosition>)>,
) {
    for (position, mut transform) in enemies.iter_mut() {
        transform.translation.x = position.x;
        transform.translation.y = position.y;
        transform.translation.z = position.z;
    }
}

/// FSM state transition system (server-only)
pub fn enemy_fsm_system(
    mut enemies: Query<(
        &Transform,
        &mut EnemyState,
        &EnemyMovement,
    ), With<Enemy>>,
    players: Query<&Transform, With<PlayerPosition>>,
) {
    for (enemy_transform, mut state, movement) in enemies.iter_mut() {
        // Find closest player
        let closest_player_dist = players.iter()
            .map(|player_transform| {
                enemy_transform.translation.distance(player_transform.translation)
            })
            .min_by(|a, b| a.partial_cmp(b).unwrap());
        
        let Some(dist) = closest_player_dist else {
            // No players, patrol
            if *state != EnemyState::Patrol {
                *state = EnemyState::Patrol;
                info!("Enemy -> Patrol (no players)");
            }
            continue;
        };
        
        // State transitions based on distance
        match *state {
            EnemyState::Patrol => {
                if dist <= movement.chase_range {
                    *state = EnemyState::Chase;
                    info!("Enemy -> Chase (player in range: {:.2})", dist);
                }
            }
            EnemyState::Chase => {
                if dist <= movement.attack_range {
                    *state = EnemyState::Attack;
                    info!("Enemy -> Attack (player close: {:.2})", dist);
                } else if dist > movement.chase_range {
                    *state = EnemyState::Patrol;
                    info!("Enemy -> Patrol (player escaped: {:.2})", dist);
                }
            }
            EnemyState::Attack => {
                if dist > movement.attack_range && dist <= movement.chase_range {
                    *state = EnemyState::Chase;
                    info!("Enemy -> Chase (player moved away: {:.2})", dist);
                } else if dist > movement.chase_range {
                    *state = EnemyState::Patrol;
                    info!("Enemy -> Patrol (player escaped: {:.2})", dist);
                }
            }
        }
    }
}

/// Flocking behavior system - calculates flocking forces (server-only)
/// This runs before movement to calculate desired velocities based on neighbors
pub fn enemy_flocking_system(
    mut enemies: Query<(
        Entity,
        &Transform,
        &EnemyState,
        &FlockingBehavior,
        &mut EnemyVelocity,
    ), With<Enemy>>,
) {
    // Collect all enemy positions and velocities for neighbor calculations
    let enemy_data: Vec<(Entity, Vec3, Vec3)> = enemies
        .iter()
        .map(|(entity, transform, _state, _flock, velocity)| {
            (entity, transform.translation, velocity.velocity)
        })
        .collect();
    
    // Calculate flocking forces for each enemy
    for (entity, transform, state, flock_params, mut velocity) in enemies.iter_mut() {
        // Only apply flocking when in Chase state (cooperative hunting)
        if *state != EnemyState::Chase {
            continue;
        }
        
        let mut cohesion = Vec3::ZERO;
        let mut alignment = Vec3::ZERO;
        let mut separation = Vec3::ZERO;
        let mut neighbor_count = 0;
        
        let my_position = transform.translation;
        
        // Calculate flocking forces based on neighbors
        for (other_entity, other_position, other_velocity) in &enemy_data {
            if *other_entity == entity {
                continue; // Skip self
            }
            
            let offset = *other_position - my_position;
            let distance = offset.length();
            
            // Only consider enemies within neighbor range
            if distance < flock_params.neighbor_range && distance > 0.01 {
                neighbor_count += 1;
                
                // Cohesion: steer towards average position of neighbors
                cohesion += *other_position;
                
                // Alignment: steer towards average velocity of neighbors
                alignment += *other_velocity;
                
                // Separation: steer away from neighbors that are too close
                // Use inverse square law - stronger repulsion when very close
                if distance < flock_params.separation_distance {
                    let strength = (flock_params.separation_distance - distance) / flock_params.separation_distance;
                    let repulsion = -offset.normalize_or_zero() * (strength * strength);
                    separation += repulsion;
                }
            }
        }
        
        // Apply flocking forces if we have neighbors
        if neighbor_count > 0 {
            // Cohesion: move towards center of mass
            cohesion = (cohesion / neighbor_count as f32 - my_position).normalize_or_zero() 
                * flock_params.cohesion_weight;
            
            // Alignment: match average velocity
            alignment = (alignment / neighbor_count as f32).normalize_or_zero() 
                * flock_params.alignment_weight;
            
            // Separation: already calculated, just apply weight
            separation = separation.normalize_or_zero() * flock_params.separation_weight;
            
            // Combine all forces
            let flocking_force = cohesion + alignment + separation;
            
            // Store the flocking influence in velocity (will be combined with movement in next system)
            velocity.velocity = flocking_force;
        } else {
            // No neighbors, reset flocking velocity
            velocity.velocity = Vec3::ZERO;
        }
    }
}

/// Enemy movement system based on current state (server-only)
pub fn enemy_movement_system(
    mut enemies: Query<(
        &mut Transform,
        &EnemyState,
        &mut PatrolData,
        &EnemyMovement,
        &EnemyVelocity,
    ), With<Enemy>>,
    players: Query<&Transform, (With<PlayerPosition>, Without<Enemy>)>,
    time: Res<Time>,
) {
    for (mut enemy_transform, state, mut patrol, movement, velocity) in enemies.iter_mut() {
        match *state {
            EnemyState::Patrol => {
                let waypoint = get_current_waypoint(&patrol);
                let direction = (waypoint - enemy_transform.translation).normalize_or_zero();
                
                if enemy_transform.translation.distance(waypoint) < 0.5 {
                    advance_waypoint(&mut patrol);
                }
                
                enemy_transform.translation += direction * movement.patrol_speed * time.delta_secs();
            }
            EnemyState::Chase | EnemyState::Attack => {
                // Find closest player
                if let Some(player_transform) = players.iter()
                    .min_by_key(|player_transform| {
                        (enemy_transform.translation.distance(player_transform.translation) * 100.0) as i32
                    })
                {
                    let direction = (player_transform.translation - enemy_transform.translation).normalize_or_zero();
                    
                    let speed = if *state == EnemyState::Chase {
                        movement.chase_speed
                    } else {
                        0.0 // Stand still when attacking
                    };
                    
                    // Combine player-seeking direction with flocking velocity
                    let mut final_direction = direction;
                    if velocity.velocity.length_squared() > 0.01 {
                        // Blend flocking (30%) with player-seeking (70%)
                        final_direction = (direction * 0.7 + velocity.velocity.normalize_or_zero() * 0.3).normalize_or_zero();
                    }
                    
                    enemy_transform.translation += final_direction * speed * time.delta_secs();
                    
                    // Make enemy look at player
                    if direction.length_squared() > 0.0 {
                        let look_target = player_transform.translation;
                        let look_dir = Vec3::new(
                            look_target.x - enemy_transform.translation.x,
                            0.0,
                            look_target.z - enemy_transform.translation.z,
                        ).normalize_or_zero();
                        
                        if look_dir.length_squared() > 0.0 {
                            enemy_transform.look_to(look_dir, Vec3::Y);
                        }
                    }
                }
            }
        }
        
        // Keep enemy on ground
        enemy_transform.translation.y = 1.0;
    }
}

/// Server-side system to sync Transform back to replicated EnemyPosition
pub fn sync_transform_to_enemy_position(
    mut enemies: Query<(&Transform, &mut EnemyPosition), With<Enemy>>,
) {
    for (transform, mut position) in enemies.iter_mut() {
        position.x = transform.translation.x;
        position.y = transform.translation.y;
        position.z = transform.translation.z;
    }
}

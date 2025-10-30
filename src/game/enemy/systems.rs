use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_replicon::prelude::*;
use crate::network::protocol::{PlayerPosition, Enemy, EnemyPosition};
use super::components::*;

/// Spawn enemies in the world (server-side)
pub fn spawn_enemies_system(
    mut commands: Commands,
    enemies: Query<&Enemy>,
) {
    // Only spawn one enemy for now
    if enemies.iter().count() > 0 {
        return;
    }
    
    let spawn_pos = Vec3::new(10.0, 1.0, 10.0);
    
    commands.spawn((
        Enemy { id: 1 },
        EnemyPosition {
            x: spawn_pos.x,
            y: spawn_pos.y,
            z: spawn_pos.z,
        },
        EnemyState::Patrol,
        PatrolData::new(spawn_pos, 5.0),
        EnemyMovement::default(),
        Transform::from_translation(spawn_pos),
        GlobalTransform::default(),
        Collider::capsule_y(0.5, 0.5),
        RigidBody::KinematicPositionBased,
        Replicated,
    ));
    
    info!("Spawned enemy at {:?}", spawn_pos);
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

/// Enemy movement system based on current state (server-only)
pub fn enemy_movement_system(
    mut enemies: Query<(
        &mut Transform,
        &EnemyState,
        &mut PatrolData,
        &EnemyMovement,
    ), With<Enemy>>,
    players: Query<&Transform, (With<PlayerPosition>, Without<Enemy>)>,
    time: Res<Time>,
) {
    for (mut enemy_transform, state, mut patrol, movement) in enemies.iter_mut() {
        match *state {
            EnemyState::Patrol => {
                let waypoint = patrol.get_current_waypoint();
                let direction = (waypoint - enemy_transform.translation).normalize_or_zero();
                
                if enemy_transform.translation.distance(waypoint) < 0.5 {
                    patrol.advance();
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
                    
                    enemy_transform.translation += direction * speed * time.delta_secs();
                    
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

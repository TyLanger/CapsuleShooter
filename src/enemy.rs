use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::prelude::*;

use crate::{health, Player};

#[derive(Component)]
pub struct Enemy;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_enemies)
            .add_event::<EnemySpawnEvent>()
            .add_system(how_to_spawn_enemies)
            .add_system(when_to_spawn_enemies)
            .add_system(enemy_movement);
    }
}

struct EnemySpawnEvent;

fn spawn_enemies(mut ev_spawn: EventWriter<EnemySpawnEvent>) {
    let num = 5;

    // spawn 5 enemies at the start.
    // when_to_spawn will also queue an enemy to spawn
    // as there are 0 at startup
    // and this event is run next frame
    // giving when_to 1 frame at the start to see 0 enemies
    for _ in 0..num {
        ev_spawn.send(EnemySpawnEvent);
    }
}

fn enemy_movement(
    q_player: Query<&Transform, (With<Player>, Without<Enemy>)>,
    mut q_enemy: Query<&mut Transform, With<Enemy>>,
    time: Res<Time>,
) {
    let player_pos = q_player.get_single().unwrap().translation;

    for mut enemy_trans in q_enemy.iter_mut() {
        let dir = player_pos - enemy_trans.translation;
        enemy_trans.translation += dir.normalize_or_zero() * 100. * time.delta_seconds();
    }
}

fn when_to_spawn_enemies(mut ev_spawn: EventWriter<EnemySpawnEvent>, q_enemies: Query<&Enemy>) {
    // check how many enemies there are
    let num_enemies = q_enemies.iter().len();

    // each frame, check if there are fewer than 2 enemies.
    // if there is, spawn one enemy
    // enemy will be spawned next frame
    // will always spawn 1 enemy on startup
    // as this sees 0 enemies before they have a chance to spawn
    if num_enemies < 2 {
        ev_spawn.send(EnemySpawnEvent);
    }
}

fn how_to_spawn_enemies(mut commands: Commands, mut ev_spawn: EventReader<EnemySpawnEvent>) {
    // send an event to get this to spawn an enemy

    // spawn an enemy for each event
    for _ in ev_spawn.iter() {
        let mut rng = rand::thread_rng();
        let spawn_pos = Vec2::new(rng.gen_range(-1.0..=1.0), rng.gen_range(-1.0..=1.0))
            .normalize_or_zero()
            .extend(0.)
            * 200.;
        commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: Color::RED,
                    custom_size: Some(Vec2::new(35., 35.)),
                    ..default()
                },
                transform: Transform::from_translation(spawn_pos),
                ..default()
            })
            .insert(Enemy)
            .insert(health::Health::new(2))
            .insert(RigidBody::Dynamic)
            .insert(LockedAxes::ROTATION_LOCKED)
            .insert(Collider::cuboid(35.0 / 2.0, 35.0 / 2.0));
    }
}

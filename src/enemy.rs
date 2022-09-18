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
            .add_system(enemy_movement);
    }
}

fn spawn_enemies(mut commands: Commands) {
    let num = 5;

    for _ in 0..num {
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

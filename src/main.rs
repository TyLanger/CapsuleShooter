use bevy::{
    prelude::*,
    render::camera::{RenderTarget, ScalingMode},
    sprite::collide_aabb::collide,
};
use rand::prelude::*;

struct MouseWorldPos(Vec2);

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Enemy;

#[derive(Component)]
struct Bullet {
    dir: Vec2,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_startup_system(spawn_player)
        .add_startup_system(spawn_enemies)
        .insert_resource(MouseWorldPos(Vec2::ZERO))
        .add_system(player_movement)
        .add_system(update_mouse_position)
        .add_system(shoot_bullet)
        .add_system(move_bullet)
        .add_system(bullet_collision)
        .run();
}

// startup systems

fn setup(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle {
        projection: OrthographicProjection {
            scaling_mode: ScalingMode::FixedHorizontal(1920.),
            scale: 1.0, // set to 2.0+ to zoom the camera out
            ..default()
        },
        ..default()
    });
}

fn spawn_player(mut commands: Commands) {
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::BLUE,
                custom_size: Some(Vec2::new(50., 50.)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::ZERO),
            ..default()
        })
        .insert(Player);
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
            .insert(Enemy);
    }
}

// systems

fn player_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut q_player: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
    let mut transform = q_player.single_mut();
    let mut move_input = Vec2::ZERO;

    if keyboard_input.pressed(KeyCode::A) {
        move_input.x = -1.;
    } else if keyboard_input.pressed(KeyCode::D) {
        move_input.x = 1.;
    }

    if keyboard_input.pressed(KeyCode::S) {
        move_input.y = -1.;
    } else if keyboard_input.pressed(KeyCode::W) {
        move_input.y = 1.;
    }

    let move_speed = 350.;
    transform.translation +=
        move_input.normalize_or_zero().extend(0.) * time.delta_seconds() * move_speed;
}

fn update_mouse_position(
    windows: Res<Windows>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    mut mouse_pos: ResMut<MouseWorldPos>,
) {
    let (camera, camera_transform) = q_camera.single();

    let win = if let RenderTarget::Window(id) = camera.target {
        windows.get(id).unwrap()
    } else {
        windows.get_primary().unwrap()
    };

    if let Some(screen_pos) = win.cursor_position() {
        let window_size = Vec2::new(win.width() as f32, win.height() as f32);

        // convert screen position [0..resolution] to ndc [-1..1] (gpu coords)
        let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;

        // matrix for undoing the projection and camera transform
        let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix().inverse();

        // use it to convert ndc to world-space coordinates
        let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));

        // reduce it to a 2D value
        let world_pos: Vec2 = world_pos.truncate();

        mouse_pos.0 = world_pos;
    }
}

fn shoot_bullet(
    mut commands: Commands,
    mouse_input: Res<Input<MouseButton>>,
    q_player: Query<&Transform, With<Player>>,
    mouse_pos: Res<MouseWorldPos>,
) {
    if mouse_input.just_pressed(MouseButton::Left) {
        let dir = Vec2::new(
            mouse_pos.0.x - q_player.single().translation.x,
            mouse_pos.0.y - q_player.single().translation.y,
        )
        .normalize_or_zero();
        commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.25, 0.25, 0.75),
                    custom_size: Some(Vec2::new(10., 20.)),
                    ..default()
                },
                transform: Transform {
                    translation: q_player.single().translation.clone(),
                    rotation: Quat::from_rotation_arc_2d(Vec2::Y, dir),
                    ..default()
                },
                ..default()
            })
            .insert(Bullet { dir });
    }
}

fn move_bullet(mut q_bullet: Query<(&mut Transform, &Bullet)>, time: Res<Time>) {
    for (mut transform, bullet) in &mut q_bullet {
        // vec2 to vec3 with extend
        transform.translation += (bullet.dir * time.delta_seconds() * 500.).extend(0.);
    }
}

fn bullet_collision(
    q_bullets: Query<(Entity, &Transform, &Sprite), With<Bullet>>,
    q_enemies: Query<(Entity, &Transform, &Sprite), With<Enemy>>,
    mut commands: Commands,
) {
    for (enemy, enemy_trans, enemy_sprite) in q_enemies.iter() {
        for (bullet, bullet_trans, bullet_sprite) in q_bullets.iter() {
            let collision = collide(
                enemy_trans.translation,
                enemy_sprite.custom_size.unwrap(),
                bullet_trans.translation,
                bullet_sprite.custom_size.unwrap(),
            );

            match collision {
                Some(_) => {
                    commands.entity(enemy).despawn();
                    commands.entity(bullet).despawn();
                }
                _ => {}
            }
        }
    }
}

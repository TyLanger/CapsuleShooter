use bevy::{prelude::*, render::camera::ScalingMode};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_startup_system(spawn_player)
        .add_system(player_movement)
        .run();
}

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

fn spawn_player(
    mut commands: Commands
) {
    commands.spawn_bundle(SpriteBundle {
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

#[derive(Component)]
struct Player;

fn player_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut q_player: Query<&mut Transform, With<Player>>,
    time: Res<Time>
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
    transform.translation += move_input.normalize_or_zero().extend(0.) * time.delta_seconds() * move_speed;
}
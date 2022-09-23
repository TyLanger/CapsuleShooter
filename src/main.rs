use bevy::{
    prelude::*,
    render::camera::{RenderTarget, ScalingMode},
};
use bevy_rapier2d::prelude::*;

mod cartridge;
mod enemy;
mod health;
mod shooting;

struct MouseWorldPos(Vec2);

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Wall;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(shooting::ShootingPlugin)
        .add_plugin(enemy::EnemyPlugin)
        .add_plugin(health::HealthPlugin)
        .add_plugin(cartridge::CartridgePlugin)
        .add_startup_system(setup)
        .add_startup_system(spawn_player)
        .add_startup_system(spawn_bounds)
        //.add_startup_system(spawn_enemies)
        .insert_resource(MouseWorldPos(Vec2::ZERO))
        .insert_resource(RapierConfiguration {
            gravity: Vec2::ZERO,
            ..default()
        })
        .add_system(player_movement)
        .add_system(update_mouse_position)
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
        .insert(Player)
        .insert(Collider::cuboid(25.0, 25.0))
        .insert(RigidBody::Dynamic)
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(shooting::Gun {
            clip_size: 2,
            shots_left: 2,
            time_between_shots: 0.3,
            reload_timer: Timer::from_seconds(2.0, true),
            state: shooting::GunState::Ready,
            damage: 1,
            bullet_lifetime: 1.0,
        })
        .insert(shooting::Shotgun)
        .insert(shooting::ShotgunGauge::new(6))
        .insert(health::Health::new(100));
}

fn spawn_bounds(mut commands: Commands) {
    build_wall(
        &mut commands,
        Vec2::new(1920., 50.),
        Vec3::new(0., 540., 0.),
    );
    build_wall(
        &mut commands,
        Vec2::new(1920., 50.),
        Vec3::new(0., -540., 0.),
    );
    build_wall(
        &mut commands,
        Vec2::new(50., 1080.),
        Vec3::new(960., 0., 0.),
    );
    build_wall(
        &mut commands,
        Vec2::new(50., 1080.),
        Vec3::new(-960., 0., 0.),
    );

    // to put commands.inserts, etc in another fn:
    // add &mut before the type in the parameters of the helper
    // and &mut before the variable when calling it
    // fn this(mut commands: Commands) like normal
    // helper(&mut commands);                       &mut here
    // fn helper(mut commands: &mut Commands)       &mut here
}

fn build_wall(mut commands: &mut Commands, size: Vec2, position: Vec3) {
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::BLACK,
                custom_size: Some(size),
                ..default()
            },
            transform: Transform::from_translation(position),
            ..default()
        })
        .insert(Collider::cuboid(size.x * 0.5, size.y * 0.5))
        .insert(RigidBody::Fixed)
        .insert(Wall);
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

use bevy::{
    prelude::*,
    render::camera::{RenderTarget, ScalingMode},
    sprite::collide_aabb::collide,
};
use bevy_rapier2d::prelude::*;
use rand::prelude::*;

struct MouseWorldPos(Vec2);

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Enemy;

#[derive(Component)]
struct Bullet {
    dir: Vec2,
    lifetime: Timer,
}

impl Bullet {
    pub fn new(dir: Vec2) -> Self {
        Self {
            dir,
            lifetime: Timer::from_seconds(3.0, false),
        }
    }
}

struct BulletHitEvent {
    pos: Vec2,
}

#[derive(Component)]
struct Gun {
    clip_size: u32,
    shots_left: u32,
    time_between_shots: f32,
    reload_timer: Timer,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_startup_system(setup)
        .add_startup_system(spawn_player)
        .add_startup_system(spawn_enemies)
        .insert_resource(MouseWorldPos(Vec2::ZERO))
        .insert_resource(RapierConfiguration {
            gravity: Vec2::ZERO,
            ..default()
        })
        .add_event::<BulletHitEvent>()
        .add_system(player_movement)
        .add_system(update_mouse_position)
        .add_system(shoot_bullet)
        .add_system(reload)
        .add_system(move_bullet)
        .add_system(bullet_lifetime)
        .add_system(bullet_collision_rapier)
        .add_system(bullet_event)
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
        .insert(Gun {
            clip_size: 6,
            shots_left: 6,
            time_between_shots: 0.3,
            reload_timer: Timer::from_seconds(2.0, true),
        });
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
            .insert(RigidBody::Dynamic)
            .insert(LockedAxes::ROTATION_LOCKED)
            .insert(Collider::cuboid(35.0 / 2.0, 35.0 / 2.0));
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
    mut q_player: Query<(&Transform, &mut Gun), With<Player>>,
    mouse_pos: Res<MouseWorldPos>,
    time: Res<Time>,
    mut time_of_next_shot: Local<f32>,
) {
    let (transform, mut gun) = q_player.single_mut();

    let shot_timer_ok = time.time_since_startup().as_secs_f32() > *time_of_next_shot;
    let has_shots = gun.shots_left > 0;
    let button_pressed = mouse_input.pressed(MouseButton::Left);

    if shot_timer_ok && has_shots && button_pressed {
        gun.shots_left -= 1;
        *time_of_next_shot = time.time_since_startup().as_secs_f32() + gun.time_between_shots;

        let dir = Vec2::new(
            mouse_pos.0.x - transform.translation.x,
            mouse_pos.0.y - transform.translation.y,
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
                    translation: transform.translation.clone(),
                    rotation: Quat::from_rotation_arc_2d(Vec2::Y, dir),
                    ..default()
                },
                ..default()
            })
            .insert(Bullet::new(dir))
            .insert(RigidBody::Dynamic)
            .insert(Collider::ball(5.0))
            .insert(Sensor);
    }
}

fn reload(mut q_gun: Query<&mut Gun>, time: Res<Time>) {
    let mut gun = q_gun.single_mut();

    if gun.shots_left <= 0 {
        //println!("Reloading {:?}", time.delta().as_secs_f32());
        // take some time before you refill ammo
        // this only runs when you are out of ammo
        if gun.reload_timer.tick(time.delta()).just_finished() {
            println!("Reload finished");
            gun.shots_left = gun.clip_size;
        }
    }
}

fn move_bullet(mut q_bullet: Query<(&mut Transform, &Bullet)>, time: Res<Time>) {
    for (mut transform, bullet) in &mut q_bullet {
        // vec2 to vec3 with extend
        transform.translation += (bullet.dir * time.delta_seconds() * 500.).extend(0.);
    }
}

fn bullet_lifetime(
    mut commands: Commands,
    mut q_bullet: Query<(Entity, &mut Bullet)>,
    time: Res<Time>,
) {
    for (entity, mut bullet) in &mut q_bullet {
        if bullet.lifetime.tick(time.delta()).just_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn _bullet_collision(
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

            // might want to do rapier instead
            // https://rapier.rs/docs/user_guides/bevy_plugin/getting_started_bevy

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

fn bullet_collision_rapier(
    rapier_context: Res<RapierContext>,
    q_bullets: Query<(Entity, &Transform), With<Bullet>>,
    q_enemies: Query<Entity, With<Enemy>>,
    mut commands: Commands,
    //mut w: &mut World,
    mut ev_bullet_hit: EventWriter<BulletHitEvent>,
) {
    for bullet in q_bullets.iter() {
        for enemy in q_enemies.iter() {
            // loop over every bullet and every enemy looking for pairs
            if rapier_context.intersection_pair(bullet.0, enemy) == Some(true) {
                ev_bullet_hit.send(BulletHitEvent {
                    pos: bullet.1.translation.truncate(),
                });

                commands.entity(bullet.0).despawn();
                commands.entity(enemy).despawn();
            }
        }

        // check all the things the bullet has hit
        // I think this requires 1 thing to be Sensor
        // like unity OnTriggerEnter

        // having trouble with events and the world
        // can't pass a writer and the world both as parameters
        // can use SystemStates to get around it maybe?
        // but passing mut world into this system gives me weird errors I can't read

        // for (collider1, collider2, intersecting) in rapier_context.intersections_with(bullet.0) {
        //     // check if they are actually intersecting
        //     if intersecting {
        //         // they aren't in a specific order
        //         // figure out which one might be the enemy
        //         let enemy_collider = if collider1 == bullet.0 {
        //             collider2
        //         } else {
        //             collider1
        //         };

        //         let mut state: SystemState<
        //             EventWriter<BulletHitEvent>

        //             > = SystemState::new(&mut w);

        //         let ev_bullet_hit = state.get_mut(&mut w);

        //         // try to find an enemy component
        //         let enemy_component = w.entity(enemy_collider).get::<Enemy>();

        //         // if an enemy component exists, destroy bullet and enemy
        //         match enemy_component {
        //             Some(_) => {
        //                 // w.send_event(BulletHitEvent(
        //                 //     bullet.1.translation.truncate()
        //                 // ));
        //                 ev_bullet_hit.send(BulletHitEvent(
        //                     bullet.1.translation.truncate()
        //                 ));
        //                 commands.entity(collider1).despawn();
        //                 commands.entity(collider2).despawn();
        //             }
        //             _ => {}
        //         }
        //     }
        // }
    }
}

fn bullet_event(mut ev_bullet_hit: EventReader<BulletHitEvent>) {
    for hit in ev_bullet_hit.iter() {
        eprintln!("Bullet hit at {:?}", hit.pos);
    }
}

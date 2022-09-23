use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{cartridge::Cartridge, enemy::Enemy, health::Health, MouseWorldPos, Player, Wall};

pub struct ShootingPlugin;

impl Plugin for ShootingPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<BulletHitEvent>()
            .add_event::<ShotgunBulletEndEvent>()
            .add_event::<ImmediateReloadEvent>()
            .add_system(shoot_bullet)
            .add_system(reload)
            .add_system(immediate_reload)
            .add_system(move_bullet)
            .add_system(bullet_lifetime)
            .add_system(bullet_collision_rapier)
            .add_system(bullet_event)
            .add_system(shotgun_event)
            .add_system(shotgun_check_shots)
            .add_system(shotgun_check_gauge);
    }
}

#[derive(Component)]
struct Bullet {
    dir: Vec2,
    lifetime: Timer,
    damage: u32,
}

impl Bullet {
    pub fn new(dir: Vec2, lifetime: f32, damage: u32) -> Self {
        Self {
            dir,
            lifetime: Timer::from_seconds(lifetime, false),
            damage,
        }
    }

    // pub fn new_with_damage(dir: Vec2, damage: u32) -> Self {
    //     // this doesn't seem to be the right way to do this...
    //     // am I supposed to use ..default()???
    //     // or chain it like:
    //     // would change_damage be -> &mut self??
    //     // Bullet::new(dir).change_damage(new_damage)
    //     let mut b = Bullet::new(dir);
    //     b.damage = damage;
    //     b
    // }

    fn change_damage(mut self, damage: u32) -> Self {
        // allows chaining
        // new(dir).change_damage(5)
        // new().change_x().change_y() might be the accepted pattern
        self.damage = damage;
        self
    }
}

pub struct BulletHitEvent {
    pos: Vec2,
}

#[derive(Component)]
pub struct Gun {
    pub clip_size: u32,
    pub shots_left: u32,
    pub time_between_shots: f32,
    pub reload_timer: Timer,
    pub state: GunState,
    pub damage: u32,
    pub bullet_lifetime: f32,
}

// Ready when you have bullets and aren't waiting for time between shots
// switch to reloading when you run out of ammo
// shooting when you click and while waiting for time between shots?
// is this necessary?
// shooting might also be useful for slowing you or restricting your aim?
#[derive(PartialEq)]
pub enum GunState {
    Ready,
    Reloading,
    Shooting,
}

struct ShotgunBulletEndEvent {
    side: BulletSide,
    shot_number: u32,
    reason: BulletEndReason,
}

enum BulletEndReason {
    HitEnemy,
    HitWall,
    Expired,
}

struct ImmediateReloadEvent;

#[derive(Component)]
pub struct Shotgun;

#[derive(Component)]
struct ShotgunBullet {
    side: BulletSide,
    shot_number: u32,
}

#[derive(Debug, Clone, Copy)]
enum BulletSide {
    Left,
    Right,
}

// component struct ShotgunGauge
// vec![num_bullets] of HitPair
// struct HitPair
// left: Option<bool>
// right: Option<bool>
// get_both_hit -> Option<bool>
// none if still waiting
// query<Gauge>
// eventReader
// when event happens, add to gauge
#[derive(Component)]
pub struct ShotgunGauge {
    hit_pairs: Vec<HitPair>,
}

impl ShotgunGauge {
    pub fn new(size: usize) -> Self {
        let mut gauge = ShotgunGauge {
            hit_pairs: Vec::with_capacity(size),
        };
        for _ in 0..size {
            gauge.hit_pairs.push(HitPair::new());
        }
        gauge
    }
}

struct HitPair {
    left: Option<bool>,
    right: Option<bool>,
}

impl HitPair {
    fn new() -> Self {
        HitPair {
            left: None,
            right: None,
        }
    }
}

fn shoot_bullet(
    mut commands: Commands,
    mouse_input: Res<Input<MouseButton>>,
    mut q_player: Query<
        (
            &Transform,
            &mut Gun,
            Option<&Shotgun>,
            Option<&mut ShotgunGauge>,
            Option<&mut Cartridge>,
        ),
        With<Player>,
    >,
    mouse_pos: Res<MouseWorldPos>,
    time: Res<Time>,
    mut time_of_next_shot: Local<f32>,
) {
    let (transform, mut gun, shotgun, gauge, cart) = q_player.single_mut();

    let shot_timer_ok = time.time_since_startup().as_secs_f32() > *time_of_next_shot;
    let has_shots = gun.shots_left > 0;
    let button_pressed = mouse_input.pressed(MouseButton::Left);

    if shot_timer_ok && has_shots && button_pressed {
        *time_of_next_shot = time.time_since_startup().as_secs_f32() + gun.time_between_shots;

        let dir = Vec2::new(
            mouse_pos.0.x - transform.translation.x,
            mouse_pos.0.y - transform.translation.y,
        )
        .normalize_or_zero();

        // do more damage if you have a cart attached
        let damage = if let Some(_) = cart { 2 } else { gun.damage };

        if let Some(_shotgun) = shotgun {
            // shoot like a shotgun
            // degress to radians
            // pi / 180 = 0.0174533
            let left_dir = Quat::mul_vec3(Quat::from_rotation_z(6. * 0.0174533), dir.extend(0.0));
            let right_dir = Quat::mul_vec3(Quat::from_rotation_z(-6. * 0.0174533), dir.extend(0.0));

            // reset the tracking on this shot number
            if let Some(mut gauge) = gauge {
                gauge.hit_pairs[(gun.clip_size - gun.shots_left) as usize] = HitPair {
                    left: None,
                    right: None,
                };
            }

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
                .insert(Bullet::new(
                    left_dir.truncate(),
                    gun.bullet_lifetime,
                    damage,
                ))
                .insert(ShotgunBullet {
                    side: BulletSide::Left,
                    shot_number: gun.clip_size - gun.shots_left,
                })
                .insert(RigidBody::Dynamic)
                .insert(Collider::ball(5.0))
                .insert(Sensor);

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
                .insert(Bullet::new(
                    right_dir.truncate(),
                    gun.bullet_lifetime,
                    damage,
                ))
                .insert(ShotgunBullet {
                    side: BulletSide::Right,
                    shot_number: gun.clip_size - gun.shots_left,
                })
                .insert(RigidBody::Dynamic)
                .insert(Collider::ball(5.0))
                .insert(Sensor);
        } else {
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
                .insert(Bullet::new(dir, gun.bullet_lifetime, damage))
                .insert(RigidBody::Dynamic)
                .insert(Collider::ball(5.0))
                .insert(Sensor);
        }

        gun.shots_left -= 1;
        if gun.shots_left <= 0 {
            gun.state = GunState::Reloading;
            //gun.reload_timer = Timer::from_seconds(duration, repeating)
        }
    }
}

fn reload(mut q_gun: Query<&mut Gun>, time: Res<Time>) {
    let mut gun = q_gun.single_mut();

    if gun.state == GunState::Reloading {
        //if gun.shots_left <= 0 {
        //println!("Reloading {:?}", time.delta().as_secs_f32());
        // take some time before you refill ammo
        // this only runs when you are out of ammo
        if gun.reload_timer.tick(time.delta()).just_finished() {
            println!("Reload finished");
            gun.shots_left = gun.clip_size;
            gun.state = GunState::Ready;
        }
    }
}

fn immediate_reload(mut q_gun: Query<&mut Gun>, mut ev_reload: EventReader<ImmediateReloadEvent>) {
    for _ in ev_reload.iter() {
        let mut gun = q_gun.single_mut();

        gun.shots_left = gun.clip_size;
        gun.state = GunState::Ready;
    }
}

fn move_bullet(mut q_bullet: Query<(&mut Transform, &Bullet)>, time: Res<Time>) {
    for (mut transform, bullet) in &mut q_bullet {
        // vec2 to vec3 with extend
        transform.translation += (bullet.dir * time.delta_seconds() * 700.).extend(0.);
    }
}

fn bullet_lifetime(
    mut commands: Commands,
    mut q_bullet: Query<(Entity, &mut Bullet, Option<&ShotgunBullet>)>,
    mut ev_shotgun_end: EventWriter<ShotgunBulletEndEvent>,
    time: Res<Time>,
) {
    for (entity, mut bullet, shotgun) in &mut q_bullet {
        if bullet.lifetime.tick(time.delta()).just_finished() {
            if let Some(shotgun) = shotgun {
                ev_shotgun_end.send(ShotgunBulletEndEvent {
                    side: shotgun.side,
                    shot_number: shotgun.shot_number,
                    reason: BulletEndReason::Expired,
                });
            }
            commands.entity(entity).despawn();
        }
    }
}

fn bullet_collision_rapier(
    rapier_context: Res<RapierContext>,
    q_bullets: Query<(Entity, &Transform, &Bullet, Option<&ShotgunBullet>)>,
    mut q_enemies: Query<(Entity, &mut Health), With<Enemy>>,
    q_walls: Query<(Entity, &Wall)>,
    mut commands: Commands,
    mut ev_bullet_hit: EventWriter<BulletHitEvent>,
    mut ev_shotgun_end: EventWriter<ShotgunBulletEndEvent>,
) {
    for bullet in q_bullets.iter() {
        for (enemy, mut hp) in q_enemies.iter_mut() {
            // loop over every bullet and every enemy looking for pairs
            if rapier_context.intersection_pair(bullet.0, enemy) == Some(true) {
                if let Some(shotgun) = bullet.3 {
                    ev_shotgun_end.send(ShotgunBulletEndEvent {
                        side: shotgun.side,
                        shot_number: shotgun.shot_number,
                        reason: BulletEndReason::HitEnemy,
                    });
                }

                ev_bullet_hit.send(BulletHitEvent {
                    pos: bullet.1.translation.truncate(),
                });
                hp.take_damage(bullet.2.damage);

                commands.entity(bullet.0).despawn();
                //commands.entity(enemy).despawn();
            }
        }

        for (wall_ent, _) in q_walls.iter() {
            if rapier_context.intersection_pair(bullet.0, wall_ent) == Some(true) {

                if let Some(shotgun_bullet) = bullet.3 {
                    ev_shotgun_end.send(ShotgunBulletEndEvent {
                        side: shotgun_bullet.side,
                        shot_number: shotgun_bullet.shot_number,
                        reason: BulletEndReason::HitWall,
                    });
                }

                // BulletHitEvent
                // would go here maybe?
                // it doesn't really do anything yet.
                // I made it to test events. But it's ambiguous if a hit is an enemy or wall

                commands.entity(bullet.0).despawn();
            }
        }
    }
}

fn bullet_event(mut ev_bullet_hit: EventReader<BulletHitEvent>) {
    for hit in ev_bullet_hit.iter() {
        eprintln!("Bullet hit at {:?}", hit.pos);
    }
}

fn shotgun_event(mut ev_shotgun_hit: EventReader<ShotgunBulletEndEvent>) {
    for hit in ev_shotgun_hit.iter() {
        match hit.reason {
            BulletEndReason::HitEnemy => {
                eprintln!(
                    "Shotgun hit on {:?} side. Number: {:?}",
                    hit.side, hit.shot_number
                );
            }
            _ => (),
        }
    }
}

fn shotgun_check_shots(
    mut ev_shotgun_end: EventReader<ShotgunBulletEndEvent>,
    mut q_gauge: Query<&mut ShotgunGauge>,
) {
    let mut gauge = q_gauge.single_mut();

    for ev in ev_shotgun_end.iter() {
        match ev.reason {
            BulletEndReason::HitEnemy => match ev.side {
                BulletSide::Left => {
                    gauge.hit_pairs[ev.shot_number as usize].left = Some(true);
                }
                BulletSide::Right => {
                    gauge.hit_pairs[ev.shot_number as usize].right = Some(true);
                }
            },
            BulletEndReason::Expired | BulletEndReason::HitWall => match ev.side {
                BulletSide::Left => {
                    gauge.hit_pairs[ev.shot_number as usize].left = Some(false);
                }
                BulletSide::Right => {
                    gauge.hit_pairs[ev.shot_number as usize].right = Some(false);
                }
            },
        }
    }
}

fn shotgun_check_gauge(
    mut q_gauge: Query<&mut ShotgunGauge>,
    mut ev_reload: EventWriter<ImmediateReloadEvent>,
) {
    let mut gauge = q_gauge.single_mut();

    for (i, pair) in gauge.hit_pairs.iter_mut().enumerate() {
        // check if both have something
        if let Some(left) = pair.left {
            if let Some(right) = pair.right {
                // for 2 bools, only 4 options

                // I want this to trigger an extra shot,
                // but reload time is 2s
                // and bullets can travel for 1s
                // so getting an extra ammo could be fairly delayed based on how long it takes to hit
                // and you'd already be halfway through a reload
                // could be +1 ammo on the next reload. Or quick-reload. Half time or immediate
                // 0.3s between shots
                // shot, 0.3, shot, 0.3, shot (immediate best case)
                // shot, 0.3, shot, 1.0, shot (immediate worst case or half)
                // shot, 0.3, shot, 2.0, shot (normal reload)
                if left && right {
                    println!("Both hit. Shot: {:?}", i);
                    ev_reload.send(ImmediateReloadEvent);
                } else if !left && !right {
                    println!("Both missed. Shot: {:?}", i);
                } else if left {
                    println!("Only left hit. Shot: {:?}", i);
                } else {
                    println!("Only right hit. Shot: {:?}", i);
                }

                // done once, now clean up the hit_pairs so it doesn't print forever
                *pair = HitPair::new();
            }
        }
    }
}

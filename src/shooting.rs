use bevy::{prelude::*, sprite::collide_aabb::collide};
use bevy_rapier2d::prelude::*;

use crate::{cartridge::Cartridge, enemy::Enemy, health::Health, MouseWorldPos, Player};

pub struct ShootingPlugin;

impl Plugin for ShootingPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<BulletHitEvent>()
            .add_event::<ShotgunBulletHitEvent>()
            .add_event::<ShotgunBulletExpireEvent>()
            .add_system(shoot_bullet)
            .add_system(reload)
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
    pub fn new(dir: Vec2) -> Self {
        Self {
            dir,
            lifetime: Timer::from_seconds(3.0, false),
            damage: 1,
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
}

pub struct ShotgunBulletHitEvent {
    side: BulletSide,
    shot_number: u32,
}

struct ShotgunBulletExpireEvent {
    side: BulletSide,
    shot_number: u32,
}

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
        let damage = if let Some(_) = cart { 2 } else { 1 };

        if let Some(_shotgun) = shotgun {
            // shoot like a shotgun
            // degress to radians
            // pi / 180 = 0.0174533
            let left_dir = Quat::mul_vec3(Quat::from_rotation_z(8. * 0.0174533), dir.extend(0.0));
            let right_dir = Quat::mul_vec3(Quat::from_rotation_z(-8. * 0.0174533), dir.extend(0.0));

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
                .insert(Bullet::new(left_dir.truncate()).change_damage(damage))
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
                .insert(Bullet::new(right_dir.truncate()).change_damage(damage))
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
                .insert(Bullet::new(dir).change_damage(damage))
                .insert(RigidBody::Dynamic)
                .insert(Collider::ball(5.0))
                .insert(Sensor);
        }

        gun.shots_left -= 1;
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
    mut q_bullet: Query<(Entity, &mut Bullet, Option<&ShotgunBullet>)>,
    mut ev_shotgun_expire: EventWriter<ShotgunBulletExpireEvent>,
    time: Res<Time>,
) {
    for (entity, mut bullet, shotgun) in &mut q_bullet {
        if bullet.lifetime.tick(time.delta()).just_finished() {
            if let Some(shotgun) = shotgun {
                ev_shotgun_expire.send(ShotgunBulletExpireEvent {
                    side: shotgun.side,
                    shot_number: shotgun.shot_number,
                });
            }
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
    q_bullets: Query<(Entity, &Transform, &Bullet, Option<&ShotgunBullet>)>,
    mut q_enemies: Query<(Entity, &mut Health), With<Enemy>>,
    mut commands: Commands,
    //mut w: &mut World,
    mut ev_bullet_hit: EventWriter<BulletHitEvent>,
    mut ev_shotgun_hit: EventWriter<ShotgunBulletHitEvent>,
) {
    for bullet in q_bullets.iter() {
        for (enemy, mut hp) in q_enemies.iter_mut() {
            // loop over every bullet and every enemy looking for pairs
            if rapier_context.intersection_pair(bullet.0, enemy) == Some(true) {
                if let Some(shotgun) = bullet.3 {
                    //println!("Hit on side: {:?}", shotgun.side);
                    ev_shotgun_hit.send(ShotgunBulletHitEvent {
                        side: shotgun.side,
                        shot_number: shotgun.shot_number,
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

fn shotgun_event(mut ev_shotgun_hit: EventReader<ShotgunBulletHitEvent>) {
    for hit in ev_shotgun_hit.iter() {
        eprintln!(
            "Shotgun hit on {:?} side. Number: {:?}",
            hit.side, hit.shot_number
        );
    }
}

fn shotgun_check_shots(
    mut ev_shotgun_hit: EventReader<ShotgunBulletHitEvent>,
    mut ev_shotgun_expire: EventReader<ShotgunBulletExpireEvent>,
    mut q_gauge: Query<&mut ShotgunGauge>,
) {
    let mut gauge = q_gauge.single_mut();

    for hit in ev_shotgun_hit.iter() {
        match hit.side {
            BulletSide::Left => {
                gauge.hit_pairs[hit.shot_number as usize].left = Some(true);
            }
            BulletSide::Right => {
                gauge.hit_pairs[hit.shot_number as usize].right = Some(true);
            }
        }
    }

    for expire in ev_shotgun_expire.iter() {
        match expire.side {
            BulletSide::Left => {
                gauge.hit_pairs[expire.shot_number as usize].left = Some(false);
            }
            BulletSide::Right => {
                gauge.hit_pairs[expire.shot_number as usize].right = Some(false);
            }
        }
    }
    // when something hits or expires
    // set the state
    // then wait for the second one
    // when we have them both,
    // check if both hit or exactly one or whatever we're doing
    // need to do this for each generation
    // reset on reload? Reset on a new shot
    // might not work right
    // can i guarantee bullets are done when you finish reload?
    // probably not.
    // but each shot is independant. Shot 0 is different than shot 4.
    // you'd have to cycle through all shots, reload and shoot again
    // all before the og shots terminated to conflict

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

    // got version 1 to work
    // but v2 might be better?

    // Version 2
    // bullets are given ref to each other
    // first bullet to hit sends message to other bullet
    // if they expire, no message bc they expire at the same time
    // if other.hit
    // event.send(both hit)
    // event.send(one hit)
}

fn shotgun_check_gauge(mut q_gauge: Query<&mut ShotgunGauge>) {
    let mut gauge = q_gauge.single_mut();

    for (i, pair) in gauge.hit_pairs.iter_mut().enumerate() {
        // check if both have something
        if let Some(left) = pair.left {
            if let Some(right) = pair.right {
                // for 2 bools, only 4 options
                if left && right {
                    println!("Both hit. Shot: {:?}", i);
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

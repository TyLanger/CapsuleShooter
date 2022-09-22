use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::Player;

pub struct CartridgePlugin;

impl Plugin for CartridgePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_cart_pickup)
            .add_system(check_pickup);
    }
}

#[derive(Component)]
struct CartridgePickup;

#[derive(Component)]
pub struct Cartridge {
    power: usize,
}

fn spawn_cart_pickup(mut commands: Commands) {
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.2, 0.4, 0.8),
                custom_size: Some(Vec2::new(15., 15.)),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(300., 150., 0.),
                ..default()
            },
            ..default()
        })
        .insert(CartridgePickup)
        .insert(RigidBody::Dynamic)
        .insert(Collider::ball(7.5))
        .insert(Sensor);
}

fn check_pickup(
    mut commands: Commands,
    rapier_context: Res<RapierContext>,
    q_cart: Query<(Entity, &Transform), With<CartridgePickup>>,
    q_player: Query<(Entity, &Transform), With<Player>>,
) {
    let player = q_player.single();

    for cart in q_cart.iter() {
        if rapier_context.intersection_pair(cart.0, player.0) == Some(true) {
            println!("Player picked up the cartridge");
            commands.entity(cart.0).despawn();
            commands.entity(player.0).insert(Cartridge { power: 100 });
        }
    }
}

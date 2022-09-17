use std::cmp::max;
use bevy::prelude::*;

#[derive(Component)]
pub struct Health {
    max_health: u32,
    current_health: u32,
}

impl Health {
    pub fn new(hp: u32) -> Self {
        Health { 
            max_health: hp,
            current_health: hp,
        }
    }

    pub fn take_damage(&mut self, damage: u32) {
        self.current_health = max(0, self.current_health-damage);
    }
}

pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(death);
    }
}

//pub struct death_event;
// ref to entity?

fn death(
    mut commands: Commands,
    q_health: Query<(Entity, &Health)>,
) {
    for (ent, hp) in q_health.iter() {
        if hp.current_health == 0 {
            commands.entity(ent).despawn();
        }
    }
}
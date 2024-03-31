use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use std::collections::HashSet;

use crate::core::{
    enemies::{ContactEnemy, DamageEvent, Enemy, Health, Targetable},
    GameState, Movement,
};

pub struct ProjectileBehaviorsPlugin;

impl Plugin for ProjectileBehaviorsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (handle_homing, handle_spear, handle_weapon_collisions)
                .run_if(in_state(GameState::Game)),
        );
    }
}

#[derive(Component)]
pub struct ContactWeapon {
    pub pierce: i32,
    pub damage: f32,
}

#[derive(Component)]
pub struct SpearBehavior {
    pub angle: f32,
    pub speed: f32,
}

fn handle_spear(time: Res<Time>, mut spear_query: Query<(&mut Transform, &SpearBehavior)>) {
    for (mut spear_transform, spear) in spear_query.iter_mut() {
        let movement = Vec2::from_angle(spear.angle) * spear.speed * time.delta_seconds();
        spear_transform.rotation = Quat::from_rotation_z(spear.angle);
        spear_transform.translation.x += movement.x;
        spear_transform.translation.y += movement.y;
    }
}

#[derive(Component)]
pub struct HomingBehavior {
    pub acceleration: f32,
    pub collided: HashSet<Entity>,
}

fn handle_homing(
    time: Res<Time>,
    mut homing_query: Query<(&mut Movement, &mut Transform, &HomingBehavior)>,
    enemies_query: Query<
        (&Transform, Entity),
        (With<Enemy>, With<Targetable>, Without<HomingBehavior>),
    >,
) {
    for (mut homing_movement, mut homing_transform, homing) in homing_query.iter_mut() {
        let pos = homing_transform.translation.xy();

        let mut nearest_enemy_pos: Option<Vec2> = None;
        for (enemy_transform, enemy_entity) in enemies_query.iter() {
            if homing.collided.contains(&enemy_entity) {
                continue;
            };
            let enemy_pos = enemy_transform.translation.xy();
            if let Some(existing_enemy_pos) = nearest_enemy_pos {
                if existing_enemy_pos.distance(pos) > enemy_pos.distance(pos) {
                    nearest_enemy_pos = Some(enemy_pos);
                }
            } else {
                nearest_enemy_pos = Some(enemy_pos);
            }
        }

        let Some(nearest_enemy_pos) = nearest_enemy_pos else {
            continue;
        };

        let direction = (nearest_enemy_pos - pos).normalize_or_zero();

        homing_transform.rotation = Quat::from_rotation_z(direction.to_angle());

        homing_movement.velocity += direction * homing.acceleration * time.delta_seconds();
    }
}

fn handle_weapon_collisions(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut contact_weapons_query: Query<(&mut ContactWeapon, Entity, Option<&mut HomingBehavior>)>,
    mut enemy_query: Query<
        (&mut Health, Entity, Option<&mut Movement>),
        (With<Enemy>, With<Targetable>),
    >,
    mut ev_damage: EventWriter<DamageEvent>,
) {
    for collision_event in collision_events.read() {
        match collision_event {
            CollisionEvent::Started(entity_one, entity_two, _) => {
                let entities = [entity_one, entity_two];
                let mut maybe_weapons = contact_weapons_query.iter_many_mut(entities);
                let mut maybe_enemies = enemy_query.iter_many_mut(entities);

                if let (
                    Some((mut weapon, weapon_entity, homing_behavior)),
                    Some((mut enemy_health, enemy_entity, enemy_movement)),
                ) = (maybe_weapons.fetch_next(), maybe_enemies.fetch_next())
                {
                    weapon.pierce -= 1;

                    if weapon.pierce == -1 {
                        commands.entity(weapon_entity).despawn_recursive();
                    }

                    if let Some(mut homing_behavior) = homing_behavior {
                        homing_behavior.collided.insert(enemy_entity);
                    }

                    ev_damage.send(DamageEvent {
                        damage: weapon.damage,
                        entity: enemy_entity,
                    });
                };
            }
            _ => {}
        }
    }
}

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy_tweening::{lens::SpriteColorLens, Animator, EaseMethod, Tween};
use std::{collections::HashSet, time::Duration};

use crate::core::{
    enemies::{ContactEnemy, DamageEvent, Enemy, Health, Targetable},
    GameDespawn, GameState, Movement, PauseState, TweenDespawn, YSort,
};

pub struct ProjectileBehaviorsPlugin;

impl Plugin for ProjectileBehaviorsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                handle_homing,
                handle_spear,
                handle_fire,
                handle_weapon_collisions,
                handle_fire_collisions,
            )
                .run_if(in_state(GameState::Game))
                .run_if(in_state(PauseState::Running)),
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

#[derive(Component)]
pub struct BombBehavior {
    pub damage: f32,
    pub scale: f32,
}

#[derive(Component)]
pub struct FireBehavior {
    pub damage: f32,
    pub lifetime: f32,
    pub contact: HashSet<Entity>,
    pub timer: Timer,
}

fn handle_fire(
    mut commands: Commands,
    time: Res<Time>,
    mut fire_query: Query<(Entity, &mut FireBehavior)>,
    mut ev_damage: EventWriter<DamageEvent>,
) {
    for (fire_entity, mut fire) in fire_query.iter_mut() {
        fire.lifetime -= time.delta_seconds();
        fire.timer.tick(time.delta());

        if fire.timer.just_finished() {
            for enemy_entity in fire.contact.iter() {
                ev_damage.send(DamageEvent {
                    damage: fire.damage,
                    entity: *enemy_entity,
                });
            }
        }

        if fire.lifetime < 0. {
            let fade_tween = Tween::new(
                EaseMethod::Linear,
                Duration::from_secs_f32(0.5),
                SpriteColorLens {
                    start: Color::WHITE,
                    end: Color::WHITE.with_a(0.),
                },
            )
            .with_completed_event(0);

            commands
                .entity(fire_entity)
                .insert((TweenDespawn, Animator::new(fade_tween)))
                .remove::<FireBehavior>();
        }
    }
}

fn handle_spear(time: Res<Time>, mut spear_query: Query<(&mut Transform, &SpearBehavior)>) {
    for (mut spear_transform, spear) in spear_query.iter_mut() {
        let movement = Vec2::from_angle(spear.angle) * spear.speed * time.delta_seconds();
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
    asset_server: Res<AssetServer>,
    mut collision_events: EventReader<CollisionEvent>,
    mut contact_weapons_query: Query<(
        &mut ContactWeapon,
        Entity,
        Option<&mut HomingBehavior>,
        Option<&BombBehavior>,
    )>,
    mut enemy_query: Query<(Entity, &Transform), (With<Enemy>, With<Targetable>)>,
    mut ev_damage: EventWriter<DamageEvent>,
) {
    for collision_event in collision_events.read() {
        match collision_event {
            CollisionEvent::Started(entity_one, entity_two, _) => {
                let entities = [entity_one, entity_two];
                let mut maybe_weapons = contact_weapons_query.iter_many_mut(entities);
                let mut maybe_enemies = enemy_query.iter_many_mut(entities);

                if let (
                    Some((mut weapon, weapon_entity, homing_behavior, bomb_behavior)),
                    Some((enemy_entity, enemy_transform)),
                ) = (maybe_weapons.fetch_next(), maybe_enemies.fetch_next())
                {
                    weapon.pierce -= 1;

                    if weapon.pierce == -1 {
                        commands.entity(weapon_entity).despawn_recursive();
                    }

                    if let Some(mut homing_behavior) = homing_behavior {
                        homing_behavior.collided.insert(enemy_entity);
                    }

                    if let Some(bomb_behavior) = bomb_behavior {
                        commands.spawn((
                            Collider::ball(64.),
                            Sensor,
                            ActiveCollisionTypes::STATIC_STATIC,
                            ActiveEvents::COLLISION_EVENTS,
                            FireBehavior {
                                lifetime: 3.,
                                damage: bomb_behavior.damage,
                                contact: HashSet::new(),
                                timer: Timer::from_seconds(0.5, TimerMode::Repeating),
                            },
                            SpriteBundle {
                                texture: asset_server.load("sprites/projectiles/greek_fire.png"),
                                transform: Transform {
                                    translation: enemy_transform.translation,
                                    scale: bomb_behavior.scale..default(),
                                    ..default()
                                },
                                ..default()
                            },
                            YSort(-1.),
                            GameDespawn,
                        ));
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

fn handle_fire_collisions(
    mut collision_events: EventReader<CollisionEvent>,
    mut fire_query: Query<&mut FireBehavior>,
    mut enemy_query: Query<Entity, (With<Targetable>, With<Enemy>)>,
) {
    for collision_event in collision_events.read() {
        match collision_event {
            CollisionEvent::Started(entity_one, entity_two, _) => {
                let entities = [entity_one, entity_two];
                let mut maybe_fire = fire_query.iter_many_mut(entities);
                let mut maybe_enemies = enemy_query.iter_many_mut(entities);

                if let (Some((mut fire_behavior)), Some((enemy_entity))) =
                    (maybe_fire.fetch_next(), maybe_enemies.fetch_next())
                {
                    fire_behavior.contact.insert(enemy_entity);
                };
            }
            CollisionEvent::Stopped(entity_one, entity_two, _) => {
                let entities = [entity_one, entity_two];
                let mut maybe_fire = fire_query.iter_many_mut(entities);
                let mut maybe_enemies = enemy_query.iter_many_mut(entities);

                if let (Some((mut fire_behavior)), Some((enemy_entity))) =
                    (maybe_fire.fetch_next(), maybe_enemies.fetch_next())
                {
                    fire_behavior.contact.remove(&enemy_entity);
                };
            }
        }
    }
}

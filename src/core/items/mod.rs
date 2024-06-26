use std::{
    collections::{HashMap, HashSet, VecDeque},
    f32::consts::PI,
    time::Duration,
};

use bevy::{prelude::*, sprite::Anchor};
use bevy_rapier2d::prelude::*;
use bevy_tweening::{lens::SpriteColorLens, Animator, EaseMethod, Tween};
use rand::{thread_rng, Rng};

use crate::core::{GameState, PauseState};

use self::behaviors::{BombBehavior, ContactWeapon, HomingBehavior, HomingRotation, SpearBehavior};

use super::{
    enemies::{DamageEvent, Enemy, Targetable},
    player::{Leveling, Player},
    GameDespawn, IngameTime, Movement, TimedDespawn, TweenDespawn, YSort,
};

pub mod behaviors;

pub struct ItemsPlugin;

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum Item {
    Spear,
    Bow,
    GreekFire,
    PoseidonTrident,
    ZeusThunderbolt,
}

#[derive(Resource)]
pub struct Inventory(pub VecDeque<Item>);

impl Inventory {
    fn default() -> Self {
        Inventory(vec![Item::Spear].into())
    }
}

#[derive(Resource)]
pub struct ItemCooldowns(pub HashMap<Item, f32>);

impl Plugin for ItemsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Inventory::default())
            .insert_resource(ItemCooldowns(HashMap::new()))
            .add_plugins(behaviors::ProjectileBehaviorsPlugin)
            .add_systems(OnEnter(GameState::Game), (reset_cooldowns, reset_inventory))
            .add_systems(
                Update,
                trigger_weapons
                    .run_if(in_state(GameState::Game))
                    .run_if(in_state(PauseState::Running)),
            );
    }
}

pub const INVENTORY_SIZE: usize = 3;

pub fn get_item_sprite(item: &Item) -> &'static str {
    match item {
        Item::Spear => "sprites/items/spear.png",
        Item::Bow => "sprites/items/bow.png",
        Item::GreekFire => "sprites/items/greek_fire.png",
        Item::PoseidonTrident => "sprites/items/poseidon_trident.png",
        Item::ZeusThunderbolt => "sprites/items/zeus_thunderbolt.png",
    }
}

const ATTACK_RANGE: f32 = 400.;
const BOW_COOLDOWN: f32 = 0.15;
const BOW_SPRAY: f32 = PI / 6.;
const SPEAR_COOLDOWN: f32 = 1.;
const GREEK_FIRE_COOLDOWN: f32 = 1.5;
const POSEIDON_TRIDENT_COOLDOWN: f32 = 1.25;
const ZEUS_THUNDERBOLT_COOLDOWN: f32 = 0.75;

pub fn trigger_weapons(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    ingame_time: Res<IngameTime>,
    inventory: Res<Inventory>,
    mut item_cooldowns: ResMut<ItemCooldowns>,
    player_query: Query<(&Transform, &Leveling), With<Player>>,
    enemies_query: Query<(&Transform, Entity), (With<Enemy>, With<Targetable>, Without<Player>)>,
    mut ev_damage: EventWriter<DamageEvent>,
) {
    let (player_transform, player_leveling) = player_query.get_single().unwrap();
    let player_pos = player_transform.translation.xy();

    let mut nearest_enemies = enemies_query
        .iter()
        .map(|(enemy_transform, enemy_entity)| (enemy_transform.translation.xy(), enemy_entity))
        .collect::<Vec<(Vec2, Entity)>>();

    nearest_enemies.sort_by(|(pos_a, _), (pos_b, _)| {
        return pos_a
            .distance(player_pos)
            .partial_cmp(&pos_b.distance(player_pos))
            .unwrap();
    });

    let mut nearest_enemy = nearest_enemies.get(0);

    let mut item_count: HashMap<Item, i32> = HashMap::new();

    for item in inventory.0.iter() {
        let prev = item_count.entry(*item).or_insert(0);

        *prev += 1;
    }

    for (item, count) in item_count {
        match item {
            Item::Spear => {
                let Some(nearest_enemy) = nearest_enemy else {
                    continue;
                };

                let nearest_enemy_pos = nearest_enemy.0;

                if nearest_enemy_pos.distance(player_pos) > ATTACK_RANGE {
                    continue;
                };

                let last_fired = item_cooldowns.0.entry(item).or_insert(0.);

                if ingame_time.0 - *last_fired
                    < SPEAR_COOLDOWN / count as f32 / player_leveling.rate_multiplier
                {
                    continue;
                };

                *last_fired = ingame_time.0;

                let throw_angle = (nearest_enemy_pos - player_pos).to_angle();

                commands.spawn((
                    Collider::cuboid(32.0, 1.0),
                    Sensor,
                    ActiveCollisionTypes::STATIC_STATIC,
                    ActiveEvents::COLLISION_EVENTS,
                    ContactWeapon {
                        pierce: 2 + player_leveling.buff,
                        damage: 25. * player_leveling.damage_multiplier,
                    },
                    SpearBehavior {
                        angle: throw_angle,
                        speed: 1200.,
                    },
                    SpriteBundle {
                        texture: asset_server.load("sprites/projectiles/spear.png"),
                        transform: Transform {
                            translation: player_pos.extend(0.),
                            rotation: Quat::from_rotation_z(throw_angle),
                            ..default()
                        },
                        ..default()
                    },
                    GameDespawn,
                    YSort(0.),
                    TimedDespawn { delay: 3. },
                ));
            }
            Item::Bow => {
                let Some(nearest_enemy) = nearest_enemy else {
                    continue;
                };

                let nearest_enemy_pos = nearest_enemy.0;

                if nearest_enemy_pos.distance(player_pos) > ATTACK_RANGE {
                    continue;
                };

                let last_fired = item_cooldowns.0.entry(item).or_insert(0.);

                if ingame_time.0 - *last_fired < BOW_COOLDOWN / player_leveling.rate_multiplier {
                    continue;
                };

                *last_fired = ingame_time.0;

                for _ in 0..(count * (player_leveling.buff + 1)) {
                    let mut throw_angle = (nearest_enemy_pos - player_pos).to_angle();

                    let mut rng = thread_rng();

                    throw_angle += rng.gen_range((-1. * BOW_SPRAY)..BOW_SPRAY) / 2.;

                    commands.spawn((
                        Collider::cuboid(32.0, 1.0),
                        Sensor,
                        ActiveCollisionTypes::STATIC_STATIC,
                        ActiveEvents::COLLISION_EVENTS,
                        ContactWeapon {
                            pierce: 0,
                            damage: 5. * player_leveling.damage_multiplier,
                        },
                        SpearBehavior {
                            angle: throw_angle,
                            speed: 1250.,
                        },
                        SpriteBundle {
                            texture: asset_server.load("sprites/projectiles/arrow.png"),
                            transform: Transform {
                                translation: player_pos.extend(0.),
                                rotation: Quat::from_rotation_z(throw_angle),
                                ..default()
                            },
                            ..default()
                        },
                        GameDespawn,
                        YSort(0.),
                        TimedDespawn { delay: 3. },
                    ));
                }
            }
            Item::GreekFire => {
                let Some(nearest_enemy) = nearest_enemy else {
                    continue;
                };

                let nearest_enemy_pos = nearest_enemy.0;

                if nearest_enemy_pos.distance(player_pos) > ATTACK_RANGE {
                    continue;
                };

                let last_fired = item_cooldowns.0.entry(item).or_insert(0.);

                if ingame_time.0 - *last_fired
                    < GREEK_FIRE_COOLDOWN / count as f32 / player_leveling.rate_multiplier
                {
                    continue;
                };

                *last_fired = ingame_time.0;

                let throw_angle = (nearest_enemy_pos - player_pos).to_angle();

                commands.spawn((
                    Collider::ball(16.),
                    Sensor,
                    ActiveCollisionTypes::STATIC_STATIC,
                    ActiveEvents::COLLISION_EVENTS,
                    ContactWeapon {
                        pierce: 0,
                        damage: 15. * player_leveling.damage_multiplier,
                    },
                    SpearBehavior {
                        angle: throw_angle,
                        speed: 1000.,
                    },
                    BombBehavior {
                        scale: 1. + player_leveling.buff as f32 / 4.,
                        damage: 5. * player_leveling.damage_multiplier,
                    },
                    SpriteBundle {
                        texture: asset_server.load("sprites/projectiles/greek_fire_bomb.png"),
                        transform: Transform {
                            translation: player_pos.extend(0.),
                            ..default()
                        },
                        ..default()
                    },
                    YSort(0.),
                    GameDespawn,
                    TimedDespawn { delay: 3. },
                ));
            }
            Item::PoseidonTrident => {
                let Some(nearest_enemy) = nearest_enemy else {
                    continue;
                };

                let nearest_enemy_pos = nearest_enemy.0;

                if nearest_enemy_pos.distance(player_pos) > ATTACK_RANGE {
                    continue;
                };

                let last_fired = item_cooldowns.0.entry(item).or_insert(0.);

                if ingame_time.0 - *last_fired
                    < POSEIDON_TRIDENT_COOLDOWN / count as f32 / player_leveling.rate_multiplier
                {
                    continue;
                };

                *last_fired = ingame_time.0;

                let throw_angle = (nearest_enemy_pos - player_pos).to_angle();

                commands.spawn((
                    Collider::cuboid(32.0, 14.0),
                    Sensor,
                    ActiveCollisionTypes::STATIC_STATIC,
                    ActiveEvents::COLLISION_EVENTS,
                    ContactWeapon {
                        pierce: 5 + player_leveling.buff,
                        damage: 25. * player_leveling.damage_multiplier,
                    },
                    Movement {
                        velocity: Vec2::ZERO,
                        friction: 0.,
                        max_speed: 600.,
                    },
                    HomingBehavior {
                        acceleration: 6000.,
                        collided: HashSet::new(),
                    },
                    HomingRotation,
                    SpriteBundle {
                        texture: asset_server.load("sprites/projectiles/poseidon_trident.png"),
                        transform: Transform {
                            translation: player_pos.extend(0.),
                            rotation: Quat::from_rotation_z(throw_angle),
                            ..default()
                        },
                        ..default()
                    },
                    YSort(0.),
                    GameDespawn,
                ));
            }
            Item::ZeusThunderbolt => {
                let Some(nearest_enemy) = nearest_enemy else {
                    continue;
                };

                let nearest_enemy_pos = nearest_enemy.0;

                if nearest_enemy_pos.distance(player_pos) > ATTACK_RANGE {
                    continue;
                };

                let last_fired = item_cooldowns.0.entry(item).or_insert(0.);

                if ingame_time.0 - *last_fired
                    < ZEUS_THUNDERBOLT_COOLDOWN / count as f32 / player_leveling.rate_multiplier
                {
                    continue;
                };

                *last_fired = ingame_time.0;

                let mut enemy_iter = nearest_enemies.iter();

                for _ in 0..(count * (player_leveling.buff + 1)) {
                    let Some(nearest_enemy) = enemy_iter.next() else {
                        continue;
                    };

                    if nearest_enemy.0.distance(player_pos) > ATTACK_RANGE {
                        continue;
                    };

                    ev_damage.send(DamageEvent {
                        damage: 100. * player_leveling.damage_multiplier,
                        entity: nearest_enemy.1,
                    });

                    let fade_tween = Tween::new(
                        EaseMethod::Linear,
                        Duration::from_secs_f32(1.),
                        SpriteColorLens {
                            start: Color::WHITE,
                            end: Color::BLACK.with_a(0.),
                        },
                    )
                    .with_completed_event(0);

                    commands.spawn((
                        SpriteBundle {
                            texture: asset_server.load("sprites/projectiles/zeus_thunderbolt.png"),
                            transform: Transform {
                                translation: nearest_enemy.0.extend(0.),
                                ..default()
                            },
                            sprite: Sprite {
                                anchor: Anchor::BottomCenter,
                                ..default()
                            },
                            ..default()
                        },
                        Animator::new(fade_tween),
                        GameDespawn,
                        TweenDespawn,
                        YSort(0.),
                    ));
                }
            }
        }
    }
}

fn reset_cooldowns(mut item_cooldowns: ResMut<ItemCooldowns>) {
    item_cooldowns.0.clear();
}

fn reset_inventory(mut inventory: ResMut<Inventory>) {
    *inventory = Inventory::default();
}

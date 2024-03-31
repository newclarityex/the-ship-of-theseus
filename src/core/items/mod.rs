use std::collections::{HashMap, HashSet, VecDeque};

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::core::GameState;

use self::behaviors::{ContactWeapon, HomingBehavior, SpearBehavior};

use super::{
    enemies::{Enemy, Targetable},
    player::{Leveling, Player},
    IngameTime, Movement,
};

mod behaviors;

pub struct ItemsPlugin;

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum Item {
    PoseidonTrident,
    Spear,
}

#[derive(Resource)]
pub struct Inventory(pub VecDeque<Item>);

#[derive(Resource)]
pub struct ItemCooldowns(pub HashMap<Item, f32>);

impl Plugin for ItemsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Inventory(vec![Item::Spear].into()))
            .insert_resource(ItemCooldowns(HashMap::new()))
            .add_plugins(behaviors::ProjectileBehaviorsPlugin)
            .add_systems(Update, trigger_weapons.run_if(in_state(GameState::Game)));
    }
}

pub const INVENTORY_SIZE: usize = 3;

pub fn get_item_sprite(item: &Item) -> &'static str {
    match item {
        Item::PoseidonTrident { .. } => "sprites/items/poseidon_trident.png",
        Item::Spear { .. } => "sprites/items/spear.png",
    }
}

const SPEAR_COOLDOWN: f32 = 0.5;
const ATTACK_RANGE: f32 = 400.;
const POSEIDON_TRIDENT_COOLDOWN: f32 = 1.;

pub fn trigger_weapons(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    ingame_time: Res<IngameTime>,
    inventory: Res<Inventory>,
    mut item_cooldowns: ResMut<ItemCooldowns>,
    player_query: Query<(&Transform, &Leveling), With<Player>>,
    enemies_query: Query<&Transform, (With<Enemy>, With<Targetable>, Without<Player>)>,
) {
    let (player_transform, player_leveling) = player_query.get_single().unwrap();
    let player_pos = player_transform.translation.xy();

    let mut nearest_enemy_pos: Option<Vec2> = None;
    for enemy in enemies_query.iter() {
        let enemy_pos = enemy.translation.xy();
        if let Some(existing_enemy_pos) = nearest_enemy_pos {
            if existing_enemy_pos.distance(player_pos) > enemy_pos.distance(player_pos) {
                nearest_enemy_pos = Some(enemy_pos);
            }
        } else {
            nearest_enemy_pos = Some(enemy_pos);
        }
    }

    let mut item_count: HashMap<Item, i32> = HashMap::new();

    for item in inventory.0.iter() {
        let prev = item_count.entry(*item).or_insert(0);

        *prev += 1;
    }

    for (item, count) in item_count {
        match item {
            Item::PoseidonTrident => {
                let Some(nearest_enemy_pos) = nearest_enemy_pos else {
                    continue;
                };

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

                let spear_angle = (nearest_enemy_pos - player_pos).to_angle();

                commands.spawn((
                    Collider::cuboid(32.0, 14.0),
                    Sensor,
                    ActiveCollisionTypes::STATIC_STATIC,
                    ActiveEvents::COLLISION_EVENTS,
                    ContactWeapon {
                        pierce: 2 + player_leveling.pierce,
                        damage: 25. * player_leveling.damage_multiplier,
                    },
                    Movement {
                        velocity: Vec2::ZERO,
                        friction: 0.,
                        max_speed: 600.,
                    },
                    HomingBehavior {
                        acceleration: 5000.,
                        collided: HashSet::new(),
                    },
                    SpriteBundle {
                        texture: asset_server.load("sprites/projectiles/poseidon_trident.png"),
                        transform: Transform {
                            translation: player_pos.extend(0.),
                            rotation: Quat::from_rotation_z(spear_angle),
                            ..default()
                        },
                        ..default()
                    },
                ));
            }
            Item::Spear => {
                let Some(nearest_enemy_pos) = nearest_enemy_pos else {
                    continue;
                };

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

                let spear_angle = (nearest_enemy_pos - player_pos).to_angle();

                commands.spawn((
                    Collider::cuboid(32.0, 1.0),
                    Sensor,
                    ActiveCollisionTypes::STATIC_STATIC,
                    ActiveEvents::COLLISION_EVENTS,
                    ContactWeapon {
                        pierce: 0 + player_leveling.pierce,
                        damage: 15. * player_leveling.damage_multiplier,
                    },
                    SpearBehavior {
                        angle: spear_angle,
                        speed: 1000.,
                    },
                    SpriteBundle {
                        texture: asset_server.load("sprites/projectiles/spear.png"),
                        transform: Transform {
                            translation: player_pos.extend(0.),
                            rotation: Quat::from_rotation_z(spear_angle),
                            ..default()
                        },
                        ..default()
                    },
                ));
            }
        }
    }
}

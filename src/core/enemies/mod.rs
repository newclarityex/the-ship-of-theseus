use crate::core::items::behaviors::ContactWeapon;
use bevy::prelude::*;
use bevy_tweening::{lens::SpriteColorLens, Animator, EaseMethod, Tween};
use rand::{
    distributions::{Distribution, WeightedIndex},
    thread_rng, Rng,
};
use rangemap::{range_map, RangeMap};
use std::{f32::consts::PI, time::Duration};

use self::ai::{AIPlugin, ChaseAI, KrakenAI, SurroundAI};

use super::{
    player::{Player, XpGained},
    DistanceDespawn, IngameState, IngameTime, Movement, TimedDespawn, TweenDespawn, YSort,
};
use crate::core::GameState;

mod ai;
mod spawning;

pub struct EnemiesPlugin;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AIPlugin)
            .insert_resource(LastSpawn(0.))
            .insert_resource(EnemySpawnTables::default())
            .add_event::<DamageEvent>()
            .add_systems(
                Update,
                (spawn_enemies, spawn_blahaj, damage_enemies, update_xp_orbs)
                    .run_if(in_state(GameState::Game))
                    .run_if(in_state(IngameState::Playing)),
            );
    }
}

#[derive(Resource)]
struct LastSpawn(f32);

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct EnemyXp(f32);

#[derive(Component)]
pub struct Targetable;

#[derive(Event)]
pub struct DamageEvent {
    pub damage: f32,
    pub entity: Entity,
}

const SPAWN_DISTANCE: f32 = 800.;
const ENTITY_LIMIT: usize = 2500;
const BLAHAJ_SPAWN_CHANCE: f32 = 0.005;

#[derive(PartialEq, Eq, Clone)]
enum EnemyType {
    Ferris,
    Serpent,
    Siren,
    Kraken,
}

#[derive(Component)]
pub struct ContactEnemy;

#[derive(Component)]
pub struct EnemyKnockback {
    pub knockback: f32,
}

#[derive(PartialEq, Eq, Clone)]
struct EnemyRate {
    enemy_type: EnemyType,
    weight: i32,
}

#[derive(Component)]
pub struct Health {
    pub health: f32,
    pub max_health: f32,
}

#[derive(PartialEq, Eq, Clone)]
struct EnemySpawnTable {
    global_rate: i32,
    enemy_rates: Vec<EnemyRate>,
}

#[derive(Resource)]
struct EnemySpawnTables(RangeMap<i32, EnemySpawnTable>);
impl EnemySpawnTables {
    fn default() -> Self {
        EnemySpawnTables(range_map! {
            0..60 => EnemySpawnTable {
                global_rate: 1,
                enemy_rates: vec![EnemyRate { enemy_type: EnemyType::Serpent, weight: 2}, EnemyRate { enemy_type: EnemyType::Siren, weight: 1}],
            },
            60..120 => EnemySpawnTable {
                global_rate: 2,
                enemy_rates: vec![EnemyRate { enemy_type: EnemyType::Serpent, weight: 2}, EnemyRate { enemy_type: EnemyType::Siren, weight: 1}],
            },
            120..180 => EnemySpawnTable {
                global_rate: 3,
                enemy_rates: vec![EnemyRate { enemy_type: EnemyType::Serpent, weight: 10}, EnemyRate { enemy_type: EnemyType::Siren, weight: 10}, EnemyRate { enemy_type: EnemyType::Kraken, weight: 1}],
            },
            180..240 => EnemySpawnTable {
                global_rate: 5,
                enemy_rates: vec![EnemyRate { enemy_type: EnemyType::Serpent, weight: 10}, EnemyRate { enemy_type: EnemyType::Siren, weight: 10}, EnemyRate { enemy_type: EnemyType::Kraken, weight: 1}],
            },
            240..300=> EnemySpawnTable {
                global_rate: 10,
                enemy_rates: vec![EnemyRate { enemy_type: EnemyType::Serpent, weight: 5}, EnemyRate { enemy_type: EnemyType::Siren, weight: 5}, EnemyRate { enemy_type: EnemyType::Kraken, weight: 1}],
            },
            300..i32::MAX=> EnemySpawnTable {
                global_rate: 25,
                enemy_rates: vec![EnemyRate { enemy_type: EnemyType::Kraken, weight: 1}],
            }
        })
    }
}

fn spawn_enemies(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    ingame_time: Res<IngameTime>,
    player_query: Query<&Transform, With<Player>>,
    enemies_query: Query<Entity, With<Enemy>>,
    mut last_spawn: ResMut<LastSpawn>,
    enemy_spawn_tables: Res<EnemySpawnTables>,
) {
    let player_transform = player_query.get_single().unwrap();

    let enemy_spawn_table = enemy_spawn_tables.0.get(&(last_spawn.0 as i32)).unwrap();
    let enemy_weights = WeightedIndex::new(
        &enemy_spawn_table
            .enemy_rates
            .iter()
            .map(|enemy| enemy.weight)
            .collect::<Vec<i32>>(),
    )
    .unwrap();

    while last_spawn.0 < ingame_time.0 {
        last_spawn.0 += 1. / enemy_spawn_table.global_rate as f32;

        let entities = enemies_query.iter().count();
        if entities >= ENTITY_LIMIT {
            continue;
        };

        let mut rng = rand::thread_rng();

        let enemy_type = &enemy_spawn_table.enemy_rates[enemy_weights.sample(&mut rng)].enemy_type;

        let random_angle = rng.gen_range((0.)..(2. * PI));
        let spawn_position =
            Vec2::from_angle(random_angle) * SPAWN_DISTANCE + player_transform.translation.xy();

        match enemy_type {
            EnemyType::Ferris => spawning::spawn_ferris(
                &mut commands,
                &asset_server,
                &mut texture_atlas_layouts,
                spawn_position,
            ),
            EnemyType::Serpent => {
                spawning::spawn_serpent(&mut commands, &asset_server, spawn_position)
            }
            EnemyType::Siren => spawning::spawn_siren(&mut commands, &asset_server, spawn_position),
            EnemyType::Kraken => {
                spawning::spawn_kraken(&mut commands, &asset_server, spawn_position)
            }
        }
    }
}

#[derive(Component)]
struct Blahaj;
fn spawn_blahaj(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    blahaj_query: Query<&Blahaj>,
) {
    let player_transform = player_query.get_single().unwrap();
    if blahaj_query.get_single().is_ok() {
        return;
    }

    let mut rng = rand::thread_rng();

    if rng.gen_bool((BLAHAJ_SPAWN_CHANCE * time.delta_seconds()).into()) {
        let random_angle = rng.gen_range((0.)..(2. * PI));
        let spawn_position =
            Vec2::from_angle(random_angle) * SPAWN_DISTANCE + player_transform.translation.xy();

        commands.spawn((
            Blahaj,
            ContactWeapon {
                pierce: -1,
                damage: 20.,
            },
            ai::SurroundAI {
                chase_speed: 75.,
                surround_speed: 5.,
                surround_distance: 100.,
                clockwise: true,
            },
            SpriteBundle {
                transform: Transform::from_translation(spawn_position.extend(0.)),
                texture: asset_server.load("sprites/other/blahaj.png"),
                ..default()
            },
        ));
    }
}

#[derive(Component)]
struct XpOrb(f32);

const SMALL_ORB: f32 = 1.;
const BIG_ORB: f32 = 10.;

fn damage_enemies(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut ev_damage: EventReader<DamageEvent>,
    mut enemy_query: Query<(
        Entity,
        &mut Health,
        &EnemyXp,
        &Transform,
        Option<&mut Movement>,
    )>,
) {
    for event in ev_damage.read() {
        let Ok((enemy_entity, mut enemy_health, enemy_xp, enemy_transform, enemy_movement)) =
            enemy_query.get_mut(event.entity)
        else {
            continue;
        };

        enemy_health.health -= event.damage;

        if let Some(mut enemy_movement) = enemy_movement {
            enemy_movement.velocity = Vec2::ZERO;
        };

        if enemy_health.health < 0. {
            let tween = Tween::new(
                EaseMethod::Linear,
                Duration::from_secs_f32(0.5),
                SpriteColorLens {
                    start: Color::WHITE,
                    end: Color::rgba(0., 0., 0., 0.),
                },
            )
            .with_completed_event(0);

            commands
                .entity(enemy_entity)
                .remove::<Enemy>()
                .remove::<Movement>()
                .remove::<ContactEnemy>()
                .remove::<SurroundAI>()
                .remove::<ChaseAI>()
                .remove::<KrakenAI>()
                .insert((Animator::new(tween), TweenDespawn));

            let big_xp = (enemy_xp.0 / BIG_ORB) as i32;
            let small_xp = ((enemy_xp.0 / SMALL_ORB) as i32 - big_xp * (BIG_ORB as i32));
            for i in 0..big_xp as i32 {
                let mut rng = thread_rng();
                let direction = Vec2::from_angle(rng.gen_range(0.0..2.0 * PI));

                commands.spawn((
                    XpOrb(BIG_ORB),
                    SpriteBundle {
                        texture: asset_server.load("sprites/effects/big_xp.png"),
                        transform: Transform::from_translation(enemy_transform.translation),
                        ..default()
                    },
                    Movement {
                        max_speed: 1000.,
                        velocity: direction * rng.gen_range(25.0..75.0),
                        friction: 0.8,
                    },
                    YSort(0.),
                    DistanceDespawn,
                    TimedDespawn { delay: 30. },
                ));
            }
            for i in 0..small_xp as i32 {
                let mut rng = thread_rng();
                let direction = Vec2::from_angle(rng.gen_range(0.0..2.0 * PI));

                commands.spawn((
                    XpOrb(SMALL_ORB),
                    SpriteBundle {
                        texture: asset_server.load("sprites/effects/small_xp.png"),
                        transform: Transform::from_translation(enemy_transform.translation),
                        ..default()
                    },
                    Movement {
                        max_speed: 1000.,
                        velocity: direction * rng.gen_range(25.0..75.0),
                        friction: 0.8,
                    },
                    YSort(0.),
                    DistanceDespawn,
                    TimedDespawn { delay: 30. },
                ));
            }
        }
    }
}

const XP_ATTRACT_RANGE: f32 = 300.;
const XP_COLLECT_RANGE: f32 = 50.;
fn update_xp_orbs(
    mut commands: Commands,
    time: Res<Time>,
    mut xp_orb_query: Query<(&mut Movement, &Transform, &XpOrb, Entity)>,
    player_query: Query<&Transform, With<Player>>,
    mut ev_xp_gain: EventWriter<XpGained>,
) {
    let player_transform = player_query.get_single().unwrap();
    for (mut xp_orb_movement, xp_orb_transform, xp_orb, xp_orb_entity) in xp_orb_query.iter_mut() {
        let offset = player_transform.translation.xy() - xp_orb_transform.translation.xy();
        let distance = offset.length();
        if distance > XP_ATTRACT_RANGE {
            continue;
        };

        let direction = offset.normalize_or_zero();
        xp_orb_movement.velocity += direction * 1000. * time.delta_seconds();

        if distance < XP_COLLECT_RANGE {
            ev_xp_gain.send(XpGained(xp_orb.0));
            commands.entity(xp_orb_entity).despawn_recursive();
        }
    }
}

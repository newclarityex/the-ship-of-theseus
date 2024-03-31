use crate::core::{DistanceDespawn, Movement, YSort};

use super::{
    ai::{ChaseAI, KrakenAI, SirenAI, SurroundAI},
    ContactEnemy, Enemy, EnemyKnockback, EnemyXp, Health, Targetable,
};
use bevy::prelude::*;
use bevy_animations_manager::{AnimationData, AnimationsManager};
use bevy_rapier2d::prelude::*;
use rand::{thread_rng, Rng};

pub fn spawn_ferris(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
    spawn_position: Vec2,
) {
    let texture = asset_server.load("sprites/enemies/ferris.png");

    let layout = TextureAtlasLayout::from_grid(Vec2::new(64.0, 64.0), 1, 1, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    let mut animations_manager = AnimationsManager::new();

    animations_manager.load_animation(
        "alive",
        AnimationData {
            texture: texture.clone(),
            layout: texture_atlas_layout,
            frame_count: 1,
            frame_durations: vec![0],
            anchor: bevy::sprite::Anchor::Center,
        },
    );

    animations_manager.play("alive");

    commands.spawn((
        Collider::capsule_x(16.0, 14.0),
        Sensor,
        ActiveEvents::COLLISION_EVENTS,
        // ActiveCollisionTypes::STATIC_STATIC,
        Enemy,
        Health {
            health: 25.,
            max_health: 25.,
        },
        ChaseAI { acceleration: 75. },
        Movement {
            velocity: Vec2::ZERO,
            friction: 1.,
            max_speed: 125.,
        },
        SpriteSheetBundle {
            transform: Transform::from_translation(spawn_position.extend(0.)),
            texture,
            ..default()
        },
        YSort(0.),
        ContactEnemy,
        EnemyKnockback { knockback: 320. },
        animations_manager,
        Targetable,
        EnemyXp(1.),
        DistanceDespawn,
    ));
}

pub fn spawn_serpent(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    spawn_position: Vec2,
) {
    commands.spawn((
        Collider::capsule_y(16.0, 14.0),
        Sensor,
        ActiveEvents::COLLISION_EVENTS,
        Enemy,
        Health {
            health: 45.,
            max_health: 45.,
        },
        ChaseAI { acceleration: 75. },
        Movement {
            velocity: Vec2::ZERO,
            friction: 1.,
            max_speed: 200.,
        },
        SpriteBundle {
            transform: Transform::from_translation(spawn_position.extend(0.)),
            texture: asset_server.load("sprites/enemies/serpent.png"),
            ..default()
        },
        YSort(0.),
        ContactEnemy,
        EnemyKnockback { knockback: 320. },
        Targetable,
        EnemyXp(5.),
        DistanceDespawn,
    ));
}

pub fn spawn_siren(commands: &mut Commands, asset_server: &Res<AssetServer>, spawn_position: Vec2) {
    let mut rng = thread_rng();
    commands.spawn((
        Collider::capsule_x(16.0, 14.0),
        Sensor,
        ActiveEvents::COLLISION_EVENTS,
        Enemy,
        Health {
            health: 15.,
            max_health: 15.,
        },
        SurroundAI {
            chase_speed: 150.,
            surround_speed: 2.5,
            surround_distance: 250.,
            clockwise: rng.gen_bool(0.5),
        },
        Movement {
            velocity: Vec2::ZERO,
            friction: 1.,
            max_speed: 125.,
        },
        SpriteBundle {
            transform: Transform::from_translation(spawn_position.extend(0.)),
            texture: asset_server.load("sprites/enemies/siren.png"),
            ..default()
        },
        YSort(0.),
        EnemyKnockback { knockback: 320. },
        Targetable,
        EnemyXp(10.),
        DistanceDespawn,
        SirenAI {
            timer: Timer::from_seconds(3., TimerMode::Repeating),
        },
    ));
}

pub fn spawn_kraken(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    spawn_position: Vec2,
) {
    commands.spawn((
        Collider::ball(16.0),
        Sensor,
        ActiveEvents::COLLISION_EVENTS,
        Enemy,
        Health {
            health: 200.,
            max_health: 200.,
        },
        ChaseAI { acceleration: 75. },
        Movement {
            velocity: Vec2::ZERO,
            friction: 1.,
            max_speed: 75.,
        },
        SpriteBundle {
            transform: Transform::from_translation(spawn_position.extend(0.)),
            texture: asset_server.load("sprites/enemies/kraken.png"),
            ..default()
        },
        YSort(0.),
        ContactEnemy,
        EnemyKnockback { knockback: 320. },
        Targetable,
        EnemyXp(50.),
        DistanceDespawn,
        KrakenAI {
            timer: Timer::from_seconds(5., TimerMode::Repeating),
        },
    ));
}

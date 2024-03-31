use bevy::{prelude::*, transform};
use bevy_rapier2d::prelude::*;
use std::f32::consts::PI;

use crate::core::{player::Player, DistanceDespawn, GameState, Movement, TimedDespawn, YSort};

use super::ContactEnemy;

pub struct AIPlugin;

impl Plugin for AIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                handle_chase_ai,
                handle_surround_ai,
                handle_siren_ai,
                handle_kraken_ai,
                update_linear_projectiles,
            )
                .run_if(in_state(GameState::Game)),
        );
    }
}

#[derive(Component)]
pub struct ChaseAI {
    pub acceleration: f32,
}
#[derive(Component)]
pub struct SurroundAI {
    pub chase_speed: f32,
    pub surround_speed: f32,
    pub surround_distance: f32,
    pub clockwise: bool,
}

#[derive(Component)]
struct SurroundingAI {
    pub angle: f32,
}

#[derive(Component)]
pub struct SirenAI {
    pub timer: Timer,
}
#[derive(Component)]
pub struct KrakenAI {
    pub timer: Timer,
}

#[derive(Component)]
struct LinearProjectile {
    angle: f32,
    speed: f32,
}

#[derive(Component)]
pub struct HitAnimation {
    pub duration: f32,
    pub last_hit: f32,
}

fn handle_chase_ai(
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    mut chase_query: Query<(&mut Sprite, &ChaseAI, &mut Movement, &Transform), Without<Player>>,
) {
    let player_transform = player_query.get_single().unwrap();

    for (mut chase_sprite, chase_ai, mut chase_movement, chase_transform) in chase_query.iter_mut()
    {
        let direction = player_transform.translation.xy() - chase_transform.translation.xy();
        let normalized = direction.normalize_or_zero();

        if normalized.x < 0. {
            chase_sprite.flip_x = true;
        } else {
            chase_sprite.flip_x = false;
        };

        chase_movement.velocity += normalized * time.delta_seconds() * chase_ai.acceleration;
    }
}

fn handle_surround_ai(
    mut commands: Commands,
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    mut surround_query: Query<
        (
            Entity,
            &mut Transform,
            &mut Sprite,
            &SurroundAI,
            Option<&mut SurroundingAI>,
        ),
        Without<Player>,
    >,
) {
    let player_transform = player_query.get_single().unwrap();

    for (entity, mut surround_transform, mut surround_sprite, surround_ai, surrounding_ai) in
        surround_query.iter_mut()
    {
        if let Some(mut surrounding_ai) = surrounding_ai {
            let angle_diff = surround_ai.surround_speed * time.delta_seconds()
                / surround_ai.surround_distance
                * 2.
                * PI;
            surrounding_ai.angle += angle_diff;

            let old_pos = surround_transform.translation.xy();
            let new_pos = player_transform.translation.xy()
                + Vec2::from_angle(surrounding_ai.angle) * surround_ai.surround_distance;

            if player_transform.translation.x < new_pos.x {
                surround_sprite.flip_x = true;
            } else {
                surround_sprite.flip_x = false;
            };

            surround_transform.translation.x = new_pos.x;
            surround_transform.translation.y = new_pos.y;
        } else {
            let offset = player_transform.translation.xy() - surround_transform.translation.xy();
            let distance = offset.length();

            if distance > surround_ai.surround_distance {
                let movement = offset.normalize_or_zero() * surround_ai.chase_speed;

                if movement.x < 0. {
                    surround_sprite.flip_x = true;
                } else {
                    surround_sprite.flip_x = false;
                };

                surround_transform.translation += (movement * time.delta_seconds()).extend(0.);
            } else {
                commands.entity(entity).insert(SurroundingAI {
                    angle: offset.to_angle() + PI,
                });
            }
        }
    }
}

fn handle_siren_ai(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    mut surrounding_siren_query: Query<(&mut SirenAI, &Transform), With<SurroundingAI>>,
) {
    let player_transform = player_query.get_single().unwrap();

    for (mut siren, siren_transform) in surrounding_siren_query.iter_mut() {
        siren.timer.tick(time.delta());

        if !siren.timer.just_finished() {
            continue;
        }

        let direction = (player_transform.translation.xy() - siren_transform.translation.xy())
            .normalize_or_zero();
        let angle = direction.to_angle();

        commands.spawn((
            ContactEnemy,
            LinearProjectile { angle, speed: 80. },
            SpriteBundle {
                texture: asset_server.load("sprites/projectiles/siren_attack.png"),
                transform: Transform {
                    translation: siren_transform.translation,
                    rotation: Quat::from_rotation_z(angle),
                    ..default()
                },
                ..default()
            },
            TimedDespawn { delay: 10. },
            DistanceDespawn,
            YSort(0.),
            Sensor,
            Collider::ball(14.),
        ));
    }
}

const KRAKEN_WAVES: i32 = 8;
fn handle_kraken_ai(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    mut kraken_query: Query<(&mut KrakenAI, &Transform)>,
) {
    let player_transform = player_query.get_single().unwrap();

    for (mut kraken, kraken_transform) in kraken_query.iter_mut() {
        kraken.timer.tick(time.delta());

        if !kraken.timer.just_finished() {
            continue;
        }

        for i in 0..KRAKEN_WAVES {
            let rotation = (2. * PI / KRAKEN_WAVES as f32) * i as f32;

            commands.spawn((
                ContactEnemy,
                LinearProjectile {
                    angle: rotation,
                    speed: 100.,
                },
                SpriteBundle {
                    texture: asset_server.load("sprites/projectiles/kraken_wave.png"),
                    transform: Transform::from_translation(kraken_transform.translation),
                    sprite: Sprite {
                        flip_x: Vec2::from_angle(rotation).x < 0.,
                        ..default()
                    },
                    ..default()
                },
                TimedDespawn { delay: 10. },
                DistanceDespawn,
                YSort(0.),
                Sensor,
                Collider::ball(14.),
            ));
        }
    }
}

fn update_linear_projectiles(
    time: Res<Time>,
    mut projectiles_query: Query<(&mut Transform, &LinearProjectile)>,
) {
    for (mut transform, projectile) in projectiles_query.iter_mut() {
        let movement = Vec2::from_angle(projectile.angle) * projectile.speed * time.delta_seconds();
        transform.translation.x += movement.x;
        transform.translation.y += movement.y;
    }
}

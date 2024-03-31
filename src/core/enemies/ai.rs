use std::f32::consts::PI;

use bevy::prelude::*;

use crate::{
    core::{player::Player, Movement},
    GameState,
};

pub struct AIPlugin;

impl Plugin for AIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (handle_chase_ai, handle_surround_ai).run_if(in_state(GameState::Game)),
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

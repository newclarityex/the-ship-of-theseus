use bevy::prelude::*;
use bevy_tweening::{
    lens::{TextColorLens, TransformPositionLens, TransformScaleLens},
    Animator, EaseMethod, Lens, Tracks, Tween,
};
use rand::{thread_rng, Rng};
use std::time::Duration;

use crate::GameState;

use super::{
    enemies::{DamageEvent, Health},
    player::{Player, StatIncrease},
    TweenDespawn,
};

pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (handle_damage_events, handle_stat_events).run_if(in_state(GameState::Game)),
        );
    }
}

struct SplashTextLens {
    start: (f32, Vec3),
    end: (f32, Vec3),
}
impl Lens<(Transform, Text)> for SplashTextLens {
    fn lerp(&mut self, (target_transform, target_text): &mut (Transform, Text), ratio: f32) {
        let (start_opacity, start_translation) = self.start;
        let (end_opacity, end_translation) = self.end;

        let opacity = start_opacity.lerp(end_opacity, ratio);
        if let Some(section) = target_text.sections.get_mut(0) {
            section.style.color.set_a(opacity);
        }

        target_transform.translation =
            start_translation + (end_translation - start_translation) * ratio;
    }
}

fn handle_damage_events(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut ev_damage: EventReader<DamageEvent>,
    target_query: Query<&Transform, With<Health>>,
) {
    for event in ev_damage.read() {
        let Ok(transform) = target_query.get(event.entity) else {
            continue;
        };

        let pos = transform.translation.xy().extend(3.);

        let text_color = Color::rgb(1., 0.75, 0.);

        let fade_tween = Tween::new(
            EaseMethod::Linear,
            Duration::from_secs_f32(0.75),
            TextColorLens {
                start: text_color,
                end: text_color.with_a(0.),
                section: 0,
            },
        )
        .with_completed_event(0);

        let scale_tween = Tween::new(
            EaseMethod::Linear,
            Duration::from_secs_f32(0.75),
            TransformScaleLens {
                start: Vec3::splat(1.),
                end: Vec3::splat(2.),
            },
        );

        commands.spawn((
            Text2dBundle {
                transform: Transform {
                    translation: pos,
                    ..default()
                },
                text: Text::from_section(
                    format!("{:.0}", event.damage),
                    TextStyle {
                        font: asset_server.load("fonts/pixel_font.ttf"),
                        font_size: 28.,
                        color: text_color,
                    },
                ),
                ..default()
            },
            Animator::new(fade_tween),
            Animator::new(scale_tween),
            TweenDespawn,
        ));
    }
}

fn handle_stat_events(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut ev_stat_increase: EventReader<StatIncrease>,
    player_query: Query<&Transform, With<Player>>,
) {
    let player_transform = player_query.get_single().unwrap();

    for event in ev_stat_increase.read() {
        let mut pos = player_transform.translation.xy().extend(3.);
        pos.y += 50.;

        let mut rng = thread_rng();

        let mut final_pos = pos;
        final_pos.y += 150.;

        let text_color = Color::rgb(0.4, 0., 0.8);

        let fade_tween = Tween::new(
            EaseMethod::Linear,
            Duration::from_secs_f32(3.),
            TextColorLens {
                start: text_color,
                end: text_color.with_a(0.),
                section: 0,
            },
        )
        .with_completed_event(0);

        let move_tween = Tween::new(
            EaseMethod::Linear,
            Duration::from_secs_f32(3.),
            TransformPositionLens {
                start: pos,
                end: final_pos,
            },
        );

        commands.spawn((
            Text2dBundle {
                transform: Transform {
                    translation: pos,
                    ..default()
                },
                text: Text::from_section(
                    event.0.to_string(),
                    TextStyle {
                        font: asset_server.load("fonts/pixel_font.ttf"),
                        font_size: 28.,
                        color: text_color,
                    },
                ),
                ..default()
            },
            Animator::new(fade_tween),
            Animator::new(move_tween),
            TweenDespawn,
        ));
    }
}

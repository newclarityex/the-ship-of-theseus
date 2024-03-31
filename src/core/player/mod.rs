use bevy::{prelude::*, sprite::Anchor};
use bevy_rapier2d::prelude::*;
use std::time::Duration;

use crate::core::YSort;
use crate::GameState;

use super::{
    enemies::{ContactEnemy, EnemyKnockback},
    items::Inventory,
    MainCamera, Movement,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<XpGained>()
            .add_event::<LevelUp>()
            .add_event::<StatIncrease>()
            .add_systems(Startup, setup_player)
            .add_systems(
                Update,
                (
                    handle_movement,
                    update_camera.after(handle_movement),
                    handle_player_invuln,
                    handle_player_collisions,
                    handle_xp,
                )
                    .run_if(in_state(GameState::Game)),
            );
    }
}

#[derive(Event)]
pub struct XpGained(pub f32);

#[derive(Event)]
pub struct LevelUp(pub u32);

#[derive(Event)]
pub struct StatIncrease(pub String);

pub fn level_required_xp(level: u32) -> f32 {
    (45 + 5_i32.pow(level)) as f32
}

#[derive(Component)]
pub struct Leveling {
    pub level: u32,
    pub xp: f32,
    pub pierce: i32,
    pub damage_multiplier: f32,
}

const PIERCE_LEVELS: u32 = 5;
const DAMAGE_INCREASE: f32 = 0.25;

fn handle_xp(
    mut ev_xp_gained: EventReader<XpGained>,
    mut ev_level_up: EventWriter<LevelUp>,
    mut ev_stat_increase: EventWriter<StatIncrease>,
    mut leveling_query: Query<&mut Leveling>,
) {
    for event in ev_xp_gained.read() {
        let Ok(mut leveling) = leveling_query.get_single_mut() else {
            continue;
        };
        leveling.xp += event.0;
        let required_xp = level_required_xp(leveling.level);
        if leveling.xp > required_xp {
            leveling.level += 1;
            leveling.xp -= required_xp;

            ev_level_up.send(LevelUp(leveling.level));
            if leveling.level % PIERCE_LEVELS == 0 {
                leveling.pierce += 1;
                ev_stat_increase.send(StatIncrease("Pierce Increase!".into()));
            } else {
                leveling.damage_multiplier += DAMAGE_INCREASE;
                ev_stat_increase.send(StatIncrease("Damage Increase!".into()));
            }
        }
    }
}

#[derive(Component)]
pub struct Player {
    acceleration: f32,
}

#[derive(Component)]
pub struct InvulnerabilityTimer {
    timer: Timer,
}

fn setup_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    let invuln_duration = Duration::from_secs_f32(1.);

    let mut invuln_timer = Timer::new(invuln_duration, TimerMode::Once);
    invuln_timer.set_elapsed(invuln_duration);

    commands.spawn((
        Collider::capsule_x(48.0, 16.0),
        Sensor,
        ActiveCollisionTypes::STATIC_STATIC,
        ActiveEvents::COLLISION_EVENTS,
        Player { acceleration: 300. },
        Movement {
            velocity: Vec2::ZERO,
            friction: 3.,
            max_speed: 150.,
        },
        SpriteBundle {
            texture: asset_server.load("sprites/player_ship.png"),
            sprite: Sprite {
                anchor: Anchor::Custom(Vec2::new(0., -0.06)),
                ..default()
            },
            ..default()
        },
        InvulnerabilityTimer {
            timer: invuln_timer,
        },
        YSort(0.),
        Leveling {
            level: 1,
            xp: 0.,
            pierce: 0,
            damage_multiplier: 1.0,
        },
    ));
}

fn update_camera(
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
    player_query: Query<&Transform, (With<Player>, Without<MainCamera>)>,
) {
    let mut camera_transform = camera_query.get_single_mut().unwrap();
    let player_transform = player_query.get_single().unwrap();

    camera_transform.translation.x = player_transform.translation.x;
    camera_transform.translation.y = player_transform.translation.y;
}

fn handle_movement(
    mut player_query: Query<(&Player, &mut Movement, &mut Sprite)>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let mut direction = Vec2::ZERO;

    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        direction.y += 1.;
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.;
    }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        direction.x += 1.;
    }

    let normalized = direction.normalize_or_zero();

    let (player, mut player_movement, mut player_sprite) = player_query.get_single_mut().unwrap();

    if normalized.x < 0. {
        player_sprite.flip_x = true;
    } else if normalized.x > 0. {
        player_sprite.flip_x = false;
    };

    let acceleration = player.acceleration;
    player_movement.velocity += acceleration * normalized * time.delta_seconds();
}

fn handle_player_invuln(
    time: Res<Time>,
    mut player_query: Query<(&mut InvulnerabilityTimer, &mut Sprite), With<Player>>,
) {
    let (mut invuln_timer, mut sprite) = player_query.get_single_mut().unwrap();

    invuln_timer.timer.tick(time.delta());

    if invuln_timer.timer.finished() {
        sprite.color = Color::rgba(1., 1., 1., 1.);
    } else {
        sprite.color = Color::rgba(1., 1., 1., 0.6);
    }
}

fn handle_player_collisions(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut inventory: ResMut<Inventory>,
    mut player_query: Query<(&mut InvulnerabilityTimer, &mut Movement, &Transform), With<Player>>,
    enemies_query: Query<(&ContactEnemy, &Transform, Option<&EnemyKnockback>)>,
) {
    for collision_event in collision_events.read() {
        match collision_event {
            CollisionEvent::Started(entity_one, entity_two, _) => {
                let entities = [entity_one, entity_two];
                let mut maybe_enemy = enemies_query.iter_many(entities);
                let mut maybe_player = player_query.iter_many_mut(entities);

                if let (
                    Some((contact_enemy, enemy_transform, enemy_knockback)),
                    Some((mut player_invuln, mut player_movement, player_transform)),
                ) = (maybe_enemy.fetch_next(), maybe_player.fetch_next())
                {
                    if !player_invuln.timer.finished() {
                        continue;
                    };
                    if let Some(enemy_knockback) = enemy_knockback {
                        let player_pos = player_transform.translation.xy();
                        let enemy_pos = enemy_transform.translation.xy();

                        let direction = (player_pos - enemy_pos).normalize_or_zero();

                        player_movement.velocity += direction * enemy_knockback.knockback;
                    }
                    if inventory.0.len() == 0 {
                        next_game_state.set(GameState::Finished);
                    } else {
                        player_invuln.timer.reset();
                        inventory.0.pop_front();
                    }
                };
            }
            _ => {}
        }
    }
}

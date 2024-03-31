use bevy::{ecs::entity::Entities, prelude::*, render::camera::ScalingMode};
use bevy_tweening::TweenCompleted;

use self::player::Player;

mod audio;
mod effects;
mod enemies;
mod environment;
mod gui;
mod items;
mod player;

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            player::PlayerPlugin,
            environment::BackgroundPlugin,
            gui::GuiPlugin,
            items::ItemsPlugin,
            enemies::EnemiesPlugin,
            effects::EffectsPlugin,
            audio::AudioManagerPlugin,
        ))
        .insert_resource(IngameTime(0.))
        .insert_resource(GameStats::default())
        .insert_state(GameState::StartMenu)
        .insert_state(PauseState::Running)
        .add_systems(Startup, setup_camera)
        .add_systems(
            Update,
            (handle_start)
                .run_if(in_state(GameState::StartMenu).or_else(in_state(GameState::GameOver))),
        )
        .add_systems(OnEnter(GameState::Game), setup_ingame_time)
        .add_systems(
            PreUpdate,
            (update_ingame_time)
                .run_if(in_state(GameState::Game))
                .run_if(in_state(PauseState::Running)),
        )
        .add_systems(
            Update,
            (
                despawn_tween_entities,
                (handle_distance_despawn, handle_timed_despawn)
                    .run_if(in_state(GameState::Game))
                    .run_if(in_state(PauseState::Running)),
                y_sort,
                handle_pause.run_if(in_state(GameState::Game)),
            ),
        )
        .add_systems(
            PostUpdate,
            update_movement
                .run_if(in_state(GameState::Game))
                .run_if(in_state(PauseState::Running)),
        )
        .add_systems(OnEnter(GameState::Game), handle_game_cleanup);
    }
}

#[derive(Resource, Default)]
pub struct GameStats {
    enemies_killed: i32,
    items_collected: i32,
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
enum GameState {
    StartMenu,
    Game,
    GameOver,
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
enum PauseState {
    Running,
    Paused,
}

#[derive(Component)]
pub struct Movement {
    velocity: Vec2,
    friction: f32,
    max_speed: f32,
}

#[derive(Component)]
struct YSort(pub f32);

fn y_sort(mut q: Query<(&mut Transform, &YSort)>) {
    for (mut tf, ysort) in q.iter_mut() {
        tf.translation.z = ysort.0 - (1.0f32 / (1.0f32 + (2.0f32.powf(-0.01 * tf.translation.y))));
    }
}

#[derive(Resource, Debug)]
pub struct IngameTime(f32);

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct BackgroundCamera;

fn setup_camera(mut commands: Commands) {
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.scaling_mode = ScalingMode::FixedVertical(720.0);
    commands.spawn((MainCamera, camera_bundle));
    // commands.spawn((BackgroundCamera, camera_bundle));
}

fn handle_start(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    if keys.just_pressed(KeyCode::Space) {
        next_game_state.set(GameState::Game);
    }
}

fn setup_ingame_time(mut ingame_time: ResMut<IngameTime>) {
    ingame_time.0 = 0.;
}

fn update_ingame_time(time: Res<Time>, mut ingame_time: ResMut<IngameTime>) {
    ingame_time.0 += time.delta_seconds();
}

fn update_movement(time: Res<Time>, mut movement_query: Query<(&mut Movement, &mut Transform)>) {
    for (mut movement, mut transform) in movement_query.iter_mut() {
        movement.velocity = movement
            .velocity
            .lerp(Vec2::ZERO, movement.friction * time.delta_seconds());
        movement.velocity = movement.velocity.clamp_length_max(movement.max_speed);
        transform.translation.x += movement.velocity.x * time.delta_seconds();
        transform.translation.y += movement.velocity.y * time.delta_seconds();
    }
}

#[derive(Component)]
pub struct TweenDespawn;

fn despawn_tween_entities(
    mut commands: Commands,
    mut ev_tween_complete: EventReader<TweenCompleted>,
    tweens_query: Query<Entity, With<TweenDespawn>>,
) {
    for ev in ev_tween_complete.read() {
        if let Ok(entity) = tweens_query.get(ev.entity) {
            commands.entity(entity).despawn_recursive();
        }
    }
}

const DESPAWN_DISTANCE: f32 = 1024.;

#[derive(Component)]
pub struct DistanceDespawn;

fn handle_distance_despawn(
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
    despawn_query: Query<(&Transform, Entity), (With<DistanceDespawn>, Without<Player>)>,
) {
    let player_transform = player_query.get_single().unwrap();

    for (transform, entity) in despawn_query.iter() {
        let distance = (transform.translation.xy() - player_transform.translation.xy()).length();
        if distance > DESPAWN_DISTANCE {
            commands.entity(entity).despawn_recursive();
        }
    }
}

#[derive(Component)]
pub struct TimedDespawn {
    delay: f32,
}

fn handle_timed_despawn(
    mut commands: Commands,
    time: Res<Time>,
    mut despawn_query: Query<(&mut TimedDespawn, Entity)>,
) {
    for (mut timed_despawn, entity) in despawn_query.iter_mut() {
        timed_despawn.delay -= time.delta_seconds();
        if timed_despawn.delay < 0. {
            commands.entity(entity).despawn_recursive();
        }
    }
}

#[derive(Component)]
pub struct GameDespawn;

fn handle_game_cleanup(
    mut commands: Commands,
    mut despawn_query: Query<Entity, With<GameDespawn>>,
    mut ingame_time: ResMut<IngameTime>,
    mut game_stats: ResMut<GameStats>,
) {
    *game_stats = GameStats::default();
    ingame_time.0 = 0.;
    for entity in despawn_query.iter_mut() {
        commands.entity(entity).despawn_recursive();
    }
}

fn handle_pause(
    keys: Res<ButtonInput<KeyCode>>,
    pause_state: Res<State<PauseState>>,
    mut next_pause_state: ResMut<NextState<PauseState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        match pause_state.get() {
            PauseState::Paused => next_pause_state.set(PauseState::Running),
            PauseState::Running => next_pause_state.set(PauseState::Paused),
        }
    }
}

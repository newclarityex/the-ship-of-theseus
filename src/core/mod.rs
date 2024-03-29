use bevy::prelude::*;

use crate::GameState;

mod ship;

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ship::ShipPlugin)
            .add_systems(Startup, setup_camera)
            .add_systems(
                Update,
                (handle_start).run_if(in_state(GameState::StartMenu)),
            );
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn handle_start(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    if keys.just_pressed(KeyCode::Space) {
        next_game_state.set(GameState::Game);
    }
}

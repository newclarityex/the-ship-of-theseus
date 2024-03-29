use core::CorePlugin;

use bevy::prelude::*;

mod core;


#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
enum GameState {
    StartMenu,
    Game,
}


fn main() {
    App::new()
    .add_plugins(DefaultPlugins)
    .add_plugins(CorePlugin)
    .insert_state(GameState::StartMenu)
    .run();
}
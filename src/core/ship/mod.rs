use bevy::prelude::*;

use crate::GameState;

pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Game), setup_ship);
    }
}

#[derive(Component)]
struct PlayerShip;

fn setup_ship(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        PlayerShip,
        SpriteBundle {
            texture: asset_server.load("sprites/player_ship.png"),    
            ..default()
        }
    ));
}
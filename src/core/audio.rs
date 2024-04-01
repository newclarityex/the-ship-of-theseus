use std::collections::{HashMap, VecDeque};

use bevy::prelude::*;
use bevy_kira_audio::{prelude::*, AudioSource};

use crate::core::GameState;

#[derive(Resource)]
pub struct MusicChannel;

#[derive(Resource)]
pub struct MusicVolume(pub f64);

#[derive(Resource)]
pub struct SFXChannel;

#[derive(Resource)]
pub struct SFXVolume(pub f64);

pub struct SFXInstance {
    id: i32,
    timestamp: f32,
}

#[derive(Resource)]
pub struct SFXQueue(VecDeque<SFXInstance>);

pub struct AudioManagerPlugin;
impl Plugin for AudioManagerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AudioPlugin)
            .add_audio_channel::<MusicChannel>()
            .insert_resource(MusicVolume(0.25))
            .add_audio_channel::<SFXChannel>()
            .insert_resource(SFXVolume(0.075))
            .insert_resource(SFXQueue(VecDeque::new()))
            .add_systems(OnEnter(GameState::StartMenu), start_menu_music)
            // .add_systems(OnExit(GameState::StartMenu), stop_menu_music)
            .add_systems(OnEnter(GameState::Game), start_game_music)
            .add_systems(Update, (update_volume, clear_queued));
    }
}

const OVERLAP_RANGE: f32 = 0.05;

pub fn play_sfx(
    id: i32,
    timestamp: f32,
    handle: Handle<AudioSource>,
    sfx_channel: &Res<AudioChannel<SFXChannel>>,
    sfx_queue: &mut ResMut<SFXQueue>,
) {
    let last_instance = sfx_queue
        .0
        .iter()
        .rev()
        .find(|sfx| sfx.id == id && timestamp - sfx.timestamp < OVERLAP_RANGE);

    if last_instance.is_some() {
        return;
    }

    sfx_queue.0.push_back(SFXInstance { id, timestamp });

    sfx_channel.play(handle);
}

pub fn clear_queued(
    time: Res<Time>,
    sfx_channel: Res<AudioChannel<SFXChannel>>,
    mut sfx_queue: ResMut<SFXQueue>,
) {
    sfx_queue
        .0
        .retain(|sfx| (time.elapsed_seconds() - sfx.timestamp) <= OVERLAP_RANGE);
}

fn start_menu_music(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    music_channel: Res<AudioChannel<MusicChannel>>,
) {
    music_channel.stop();

    let asset_handle = asset_server.load("audio/music/menu.ogg");

    let instance_handle = music_channel.play(asset_handle).looped().handle();
}

fn start_game_music(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    music_channel: Res<AudioChannel<MusicChannel>>,
) {
    music_channel.stop();

    let asset_handle = asset_server.load("audio/music/game.ogg");

    let instance_handle = music_channel.play(asset_handle).loop_from(25.5).handle();
}

fn update_volume(
    music_channel: Res<AudioChannel<MusicChannel>>,
    music_volume: Res<MusicVolume>,
    sfx_channel: Res<AudioChannel<SFXChannel>>,
    sfx_volume: Res<SFXVolume>,
) {
    music_channel.set_volume(music_volume.0);
    sfx_channel.set_volume(sfx_volume.0);
}

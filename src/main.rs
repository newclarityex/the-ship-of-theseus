#![windows_subsystem = "windows"]

use bevy::{asset::AssetMetaCheck, prelude::*};
use bevy_animations_manager::AnimationPlugin;
use bevy_rapier2d::prelude::*;
use bevy_tweening::TweeningPlugin;
use core::CorePlugin;

mod core;

fn main() {
    App::new()
        .insert_resource(AssetMetaCheck::Never)
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .insert_resource(Msaa::Off)
        .add_plugins((AnimationPlugin, TweeningPlugin))
        .add_plugins(CorePlugin)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        // .add_plugins(RapierDebugRenderPlugin::default())
        .run();
}

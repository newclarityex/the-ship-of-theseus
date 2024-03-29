use bevy::{
    prelude::*, 
    sprite::{MaterialMesh2dBundle, Mesh2dHandle}
};

fn main() {
    App::new()
    .add_plugins(DefaultPlugins)
    .add_systems(Startup, setup)
    .run();
}


fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());

    let circle = Mesh2dHandle(meshes.add(Circle { radius: 50.0 }));

    commands.spawn(MaterialMesh2dBundle {
        mesh: circle,
        material: materials.add(Color::WHITE),
        transform: Transform::from_xyz(
            0.,
            0.,
            0.,
        ),
        ..default()
    });
}

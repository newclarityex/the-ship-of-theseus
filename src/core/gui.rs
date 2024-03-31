use bevy::prelude::*;

use crate::core::GameState;

use super::{
    items::{get_item_sprite, Inventory, INVENTORY_SIZE},
    player::{level_required_xp, Leveling},
};

pub struct GuiPlugin;

impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ItemSprites(Vec::new()))
            .add_systems(OnEnter(GameState::Game), (setup_items_gui, setup_xp_gui))
            .add_systems(
                Update,
                (update_items_gui, update_xp_gui).run_if(in_state(GameState::Game)),
            );
    }
}

#[derive(Resource)]
struct ItemSprites(Vec<Entity>);

#[derive(Component)]
struct ItemSprite;

fn setup_items_gui(mut commands: Commands, mut item_sprites: ResMut<ItemSprites>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                // flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            // bottom align
            parent
                .spawn(NodeBundle {
                    style: Style {
                        margin: UiRect::top(Val::Px(124.)),
                        column_gap: Val::Px(12.),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    for _ in 0..INVENTORY_SIZE {
                        parent
                            .spawn((NodeBundle {
                                style: Style {
                                    border: UiRect::all(Val::Px(4.)),
                                    ..default()
                                },
                                border_color: Color::rgb(0.2, 0.2, 0.2).into(),
                                background_color: Color::rgb(0.35, 0.35, 0.35).into(),
                                ..default()
                            },))
                            .with_children(|parent| {
                                let item_box = parent
                                    .spawn((
                                        ItemSprite,
                                        NodeBundle {
                                            style: Style {
                                                width: Val::Px(64.0),
                                                height: Val::Px(64.0),
                                                ..default()
                                            },
                                            background_color: Color::WHITE.into(),
                                            ..default()
                                        },
                                    ))
                                    .id();
                                item_sprites.0.push(item_box);
                            });
                    }
                });
        });
}

fn update_items_gui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut item_sprites_query: Query<(Option<&mut UiImage>, &mut Visibility), With<ItemSprite>>,
    inventory: Res<Inventory>,
    item_sprites: Res<ItemSprites>,
) {
    for (index, item_box) in item_sprites.0.iter().enumerate() {
        let item = inventory.0.get(index);

        let Ok((item_image, mut item_visibility)) = item_sprites_query.get_mut(*item_box) else {
            // Items not setup yet
            continue;
        };

        if let Some(item) = item {
            let texture: Handle<Image> = asset_server.load(get_item_sprite(item));

            *item_visibility = Visibility::Visible;

            if let Some(mut item_image) = item_image {
                item_image.texture = texture;
            } else {
                commands.entity(*item_box).insert(UiImage {
                    texture,
                    ..default()
                });
            };
        } else {
            *item_visibility = Visibility::Hidden;
            commands.entity(*item_box).remove::<UiImage>();
        };
    }
}

#[derive(Component)]
struct XpBar;

fn setup_xp_gui(mut commands: Commands, mut item_sprites: ResMut<ItemSprites>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        max_width: Val::Px(1024.),
                        padding: UiRect::all(Val::Px(24.)),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                border: UiRect::all(Val::Px(4.)),
                                width: Val::Percent(100.),
                                ..default()
                            },
                            border_color: Color::rgb(0.2, 0.2, 0.2).into(),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                XpBar,
                                NodeBundle {
                                    style: Style {
                                        width: Val::Percent(0.),
                                        height: Val::Px(16.),
                                        ..default()
                                    },
                                    background_color: Color::rgb(0., 0.8, 0.).into(),
                                    ..default()
                                },
                            ));
                        });
                });
        });
}

fn update_xp_gui(
    mut xp_bar_query: Query<&mut Style, With<XpBar>>,
    mut leveling_query: Query<&Leveling>,
) {
    let mut xp_bar_style = xp_bar_query.get_single_mut().unwrap();
    let leveling = leveling_query.get_single().unwrap();

    let required_xp = level_required_xp(leveling.level);
    xp_bar_style.width = Val::Percent(leveling.xp / required_xp * 100.);
}

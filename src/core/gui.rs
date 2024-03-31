use bevy::prelude::*;

use crate::core::{GameState, PauseState};

use super::{
    items::{get_item_sprite, Inventory, INVENTORY_SIZE},
    player::{level_required_xp, Leveling},
    GameStats, IngameTime,
};

pub struct GuiPlugin;

impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ItemSprites(Vec::new()))
            .add_systems(OnEnter(GameState::StartMenu), setup_start_menu)
            .add_systems(OnExit(GameState::StartMenu), cleanup_start_menu)
            .add_systems(OnEnter(GameState::Game), (setup_items_gui, setup_upper_gui))
            .add_systems(
                OnExit(GameState::Game),
                (cleanup_items_gui, cleanup_upper_gui),
            )
            .add_systems(OnEnter(GameState::GameOver), (setup_stats_menu))
            .add_systems(OnExit(GameState::GameOver), (cleanup_stats_menu))
            .add_systems(OnEnter(PauseState::Paused), (setup_pause_menu))
            .add_systems(OnExit(PauseState::Paused), (cleanup_pause_menu))
            .add_systems(
                Update,
                (update_items_gui, update_xp_gui, update_timer_gui)
                    .run_if(in_state(GameState::Game))
                    .run_if(in_state(PauseState::Running)),
            );
    }
}

#[derive(Resource)]
struct ItemSprites(Vec<Entity>);

#[derive(Component)]
struct ItemsContainer;

#[derive(Component)]
struct ItemSprite;

fn setup_items_gui(mut commands: Commands, mut item_sprites: ResMut<ItemSprites>) {
    commands
        .spawn((
            ItemsContainer,
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|parent| {
            // bottom align
            parent
                .spawn(NodeBundle {
                    style: Style {
                        margin: UiRect::top(Val::Px(128.)),
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
                                    border: UiRect::all(Val::Px(2.)),
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
                                                width: Val::Px(32.0),
                                                height: Val::Px(32.0),
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

fn cleanup_items_gui(
    mut commands: Commands,
    items_gui_query: Query<Entity, With<ItemsContainer>>,
    mut item_sprites: ResMut<ItemSprites>,
) {
    let Ok(items_gui) = items_gui_query.get_single() else {
        return;
    };

    commands.entity(items_gui).despawn_recursive();
    item_sprites.0.clear();
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

#[derive(Component)]
struct UpperGuiContainer;

#[derive(Component)]
struct TimerGui;

fn setup_upper_gui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            UpperGuiContainer,
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        max_width: Val::Px(1024.),
                        padding: UiRect::all(Val::Px(24.)),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(24.),
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
                    parent.spawn((
                        TimerGui,
                        TextBundle {
                            style: Style {
                                padding: UiRect::all(Val::Px(24.)),
                                ..default()
                            },
                            text: Text::from_section(
                                "0",
                                TextStyle {
                                    font: asset_server.load("fonts/pixel_font.ttf"),
                                    font_size: 56.,
                                    color: Color::WHITE,
                                },
                            ),
                            ..default()
                        },
                    ));
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

fn update_timer_gui(
    asset_server: Res<AssetServer>,
    ingame_time: Res<IngameTime>,
    mut timer_gui_query: Query<&mut Text, With<TimerGui>>,
) {
    let Ok(mut timer_gui) = timer_gui_query.get_single_mut() else {
        return;
    };

    *timer_gui = Text::from_section(
        format!("{:.0}", ingame_time.0),
        TextStyle {
            font: asset_server.load("fonts/pixel_font.ttf"),
            font_size: 56.,
            color: Color::WHITE,
        },
    );
}
fn cleanup_upper_gui(
    mut commands: Commands,
    upper_gui_query: Query<Entity, With<UpperGuiContainer>>,
) {
    let Ok(upper_gui) = upper_gui_query.get_single() else {
        return;
    };

    commands.entity(upper_gui).despawn_recursive();
}

#[derive(Component)]
struct StartMenu;

fn setup_start_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            StartMenu,
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    padding: UiRect::all(Val::Px(20.)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        height: Val::Percent(100.),
                        max_height: Val::Px(600.),
                        width: Val::Percent(100.),

                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn(NodeBundle {
                            background_color: Color::rgba(1., 1., 1., 0.7).into(),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent
                                .spawn(NodeBundle {
                                    style: Style {
                                        padding: UiRect::all(Val::Px(20.)),
                                        ..default()
                                    },
                                    ..default()
                                })
                                .with_children(|parent| {
                                    parent.spawn(TextBundle {
                                        text: Text::from_section(
                                            "The Ship of Theseus",
                                            TextStyle {
                                                font: asset_server.load("fonts/pixel_font.ttf"),
                                                font_size: 42.,
                                                color: Color::BLACK,
                                            },
                                        ),
                                        ..default()
                                    });
                                });
                        });
                    parent
                        .spawn(NodeBundle {
                            background_color: Color::rgba(1., 1., 1., 0.7).into(),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent
                                .spawn(NodeBundle {
                                    style: Style {
                                        padding: UiRect::all(Val::Px(20.)),
                                        ..default()
                                    },
                                    ..default()
                                })
                                .with_children(|parent| {
                                    parent.spawn(TextBundle {
                                        text: Text::from_section(
                                            "Press Space to Start",
                                            TextStyle {
                                                font: asset_server.load("fonts/pixel_font.ttf"),
                                                font_size: 28.,
                                                color: Color::BLACK,
                                            },
                                        ),
                                        ..default()
                                    });
                                });
                        });
                });
        });
}

fn cleanup_start_menu(mut commands: Commands, start_menu_query: Query<Entity, With<StartMenu>>) {
    let Ok(start_menu) = start_menu_query.get_single() else {
        return;
    };

    commands.entity(start_menu).despawn_recursive();
}

#[derive(Component)]
struct StatsMenu;

fn setup_stats_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    leveling_query: Query<&Leveling>,
    ingame_time: Res<IngameTime>,
    game_stats: Res<GameStats>,
) {
    let leveling = leveling_query.get_single().unwrap();
    commands
        .spawn((
            StatsMenu,
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    padding: UiRect::all(Val::Px(20.)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|parent| {
            parent
                .spawn((NodeBundle {
                    background_color: Color::rgba(1., 1., 1., 0.7).into(),
                    ..default()
                },))
                .with_children(|parent| {
                    parent
                        .spawn((NodeBundle {
                            style: Style {
                                padding: UiRect::all(Val::Px(20.)),
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                row_gap: Val::Px(20.),
                                ..default()
                            },
                            ..default()
                        },))
                        .with_children(|parent| {
                            let row_container = NodeBundle {
                                style: Style {
                                    width: Val::Percent(100.),
                                    padding: UiRect::all(Val::Px(20.)),
                                    flex_direction: FlexDirection::Row,
                                    justify_content: JustifyContent::SpaceBetween,
                                    align_items: AlignItems::Center,
                                    column_gap: Val::Px(20.),
                                    ..default()
                                },
                                ..default()
                            };
                            parent.spawn(TextBundle {
                                text: Text::from_section(
                                    format!("Survived {:.0} seconds", ingame_time.0),
                                    TextStyle {
                                        font: asset_server.load("fonts/pixel_font.ttf"),
                                        font_size: 42.,
                                        color: Color::BLACK,
                                    },
                                ),
                                ..default()
                            });
                            parent
                                .spawn((NodeBundle {
                                    style: Style {
                                        padding: UiRect::vertical(Val::Px(20.)),
                                        flex_direction: FlexDirection::Column,
                                        align_items: AlignItems::Center,
                                        row_gap: Val::Px(20.),
                                        width: Val::Percent(100.),
                                        ..default()
                                    },
                                    ..default()
                                },))
                                .with_children(|parent| {
                                    parent.spawn(row_container.clone()).with_children(|parent| {
                                        parent.spawn(TextBundle {
                                            text: Text::from_section(
                                                "Level",
                                                TextStyle {
                                                    font: asset_server.load("fonts/pixel_font.ttf"),
                                                    font_size: 28.,
                                                    color: Color::BLACK,
                                                },
                                            ),
                                            ..default()
                                        });
                                        parent.spawn(TextBundle {
                                            text: Text::from_section(
                                                format!("{}", leveling.level),
                                                TextStyle {
                                                    font: asset_server.load("fonts/pixel_font.ttf"),
                                                    font_size: 28.,
                                                    color: Color::BLACK,
                                                },
                                            ),
                                            ..default()
                                        });
                                    });
                                    parent.spawn(row_container.clone()).with_children(|parent| {
                                        parent.spawn(TextBundle {
                                            text: Text::from_section(
                                                "Foes Defeated",
                                                TextStyle {
                                                    font: asset_server.load("fonts/pixel_font.ttf"),
                                                    font_size: 28.,
                                                    color: Color::BLACK,
                                                },
                                            ),
                                            ..default()
                                        });
                                        parent.spawn(TextBundle {
                                            text: Text::from_section(
                                                format!("{}", game_stats.enemies_killed),
                                                TextStyle {
                                                    font: asset_server.load("fonts/pixel_font.ttf"),
                                                    font_size: 28.,
                                                    color: Color::BLACK,
                                                },
                                            ),
                                            ..default()
                                        });
                                    });
                                    parent.spawn(row_container.clone()).with_children(|parent| {
                                        parent.spawn(TextBundle {
                                            text: Text::from_section(
                                                "Parts Collected",
                                                TextStyle {
                                                    font: asset_server.load("fonts/pixel_font.ttf"),
                                                    font_size: 28.,
                                                    color: Color::BLACK,
                                                },
                                            ),
                                            ..default()
                                        });
                                        parent.spawn(TextBundle {
                                            text: Text::from_section(
                                                format!("{}", game_stats.items_collected),
                                                TextStyle {
                                                    font: asset_server.load("fonts/pixel_font.ttf"),
                                                    font_size: 28.,
                                                    color: Color::BLACK,
                                                },
                                            ),
                                            ..default()
                                        });
                                    });
                                    parent.spawn(row_container.clone()).with_children(|parent| {
                                        parent.spawn(TextBundle {
                                            text: Text::from_section(
                                                "Same Ship?",
                                                TextStyle {
                                                    font: asset_server.load("fonts/pixel_font.ttf"),
                                                    font_size: 28.,
                                                    color: Color::BLACK,
                                                },
                                            ),
                                            ..default()
                                        });
                                        parent.spawn(TextBundle {
                                            text: Text::from_section(
                                                "Maybe",
                                                TextStyle {
                                                    font: asset_server.load("fonts/pixel_font.ttf"),
                                                    font_size: 28.,
                                                    color: Color::BLACK,
                                                },
                                            ),
                                            ..default()
                                        });
                                    });
                                });
                            parent.spawn(TextBundle {
                                text: Text::from_section(
                                    "Press Space to Restart",
                                    TextStyle {
                                        font: asset_server.load("fonts/pixel_font.ttf"),
                                        font_size: 42.,
                                        color: Color::BLACK,
                                    },
                                ),
                                ..default()
                            });
                        });
                });
        });
}

fn cleanup_stats_menu(mut commands: Commands, stats_menu_query: Query<Entity, With<StatsMenu>>) {
    let Ok(stats_menu) = stats_menu_query.get_single() else {
        return;
    };

    commands.entity(stats_menu).despawn_recursive();
}

#[derive(Component)]
struct PauseMenu;

fn setup_pause_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            PauseMenu,
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    background_color: Color::rgba(1., 1., 1., 0.7).into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                padding: UiRect::all(Val::Px(20.)),
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn(TextBundle {
                                text: Text::from_section(
                                    "Paused",
                                    TextStyle {
                                        font: asset_server.load("fonts/pixel_font.ttf"),
                                        font_size: 42.,
                                        color: Color::BLACK,
                                    },
                                ),
                                ..default()
                            });
                        });
                });
        });
}

fn cleanup_pause_menu(mut commands: Commands, pause_menu_query: Query<Entity, With<PauseMenu>>) {
    let Ok(pause_menu) = pause_menu_query.get_single() else {
        return;
    };

    commands.entity(pause_menu).despawn_recursive();
}

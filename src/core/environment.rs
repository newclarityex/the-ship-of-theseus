use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::{
    distributions::{Distribution, WeightedIndex},
    thread_rng, Rng,
};
use rangemap::{range_map, RangeMap};
use std::{collections::HashSet, f32::consts::PI};

use crate::GameState;

use super::{
    enemies::{ContactEnemy, Enemy, EnemyKnockback},
    items::{get_item_sprite, Inventory, Item, INVENTORY_SIZE},
    player::Player,
    IngameTime, YSort,
};

pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CurrentChunks(HashSet::new()))
            // .add_systems(OnEnter(GameState::Game), setup_background)
            .insert_resource(ItemSpawnTables::default())
            // .add_systems(OnExit(GameState::Game), cleanup_chunks)
            .add_systems(Update, (update_chunks, update_offset))
            .add_systems(
                Update,
                (handle_item_pickups).run_if(in_state(GameState::Game)),
            );
    }
}

const TILE_SIZE: f32 = 32.;
const CHUNK_SIZE: f32 = TILE_SIZE * 10.;
const RENDER_DISTANCE_Y: u16 = 4;
const RENDER_DISTANCE_X: u16 = 4;

#[derive(Resource, Debug)]
pub struct CurrentChunks(HashSet<IVec2>);

#[derive(Component)]
struct Chunk {
    pos: IVec2,
}

#[derive(Component)]
struct AnimateOffset {
    angle: f32,
    speed: f32,
}

#[derive(PartialEq, Eq, Clone)]
struct ItemRate {
    item_type: Item,
    weight: i32,
}

#[derive(PartialEq, Eq, Clone)]
struct ItemSpawnTable {
    item_rates: Vec<ItemRate>,
}

#[derive(Component)]
struct ItemPickup {
    item_type: Item,
}

#[derive(Resource)]
struct ItemSpawnTables(RangeMap<i32, ItemSpawnTable>);
impl ItemSpawnTables {
    fn default() -> Self {
        ItemSpawnTables(range_map! {
            0..30 => ItemSpawnTable {
                item_rates: vec![ItemRate { item_type: Item::Spear, weight: 1 }],
            },
            30..i32::MAX => ItemSpawnTable {
                item_rates: vec![ItemRate { item_type: Item::Spear, weight: 9 }, ItemRate { item_type: Item::PoseidonTrident, weight: 1 }],
            }
        })
    }
}

// Per Chunk
const ITEM_RATE: f32 = 0.1;

// Per Chunk
const ROCK_RATE: f32 = 0.65;

fn get_chunks_needed(
    current_chunk: &IVec2,
    render_distance_x: u16,
    render_distance_y: u16,
) -> Vec<IVec2> {
    let render_distance_x = i32::from(render_distance_x);
    let render_distance_y = i32::from(render_distance_y);
    let mut chunks_needed: Vec<IVec2> = Vec::new();
    for x in (current_chunk.x - render_distance_x)..=(current_chunk.x + render_distance_x) {
        for y in (current_chunk.y - render_distance_y)..=(current_chunk.y + render_distance_y) {
            chunks_needed.push(IVec2::new(x, y));
        }
    }

    chunks_needed
}

fn update_chunks(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player_query: Query<&Transform, With<Player>>,
    chunks_query: Query<(Entity, &Chunk)>,
    mut current_chunks: ResMut<CurrentChunks>,
    ingame_time: Res<IngameTime>,
    item_spawn_tables: Res<ItemSpawnTables>,
) {
    let item_spawn_table = item_spawn_tables.0.get(&(ingame_time.0 as i32)).unwrap();
    let item_weights = WeightedIndex::new(
        &item_spawn_table
            .item_rates
            .iter()
            .map(|item| item.weight)
            .collect::<Vec<i32>>(),
    )
    .unwrap();

    // Create new chunks
    let Ok(transform) = player_query.get_single() else {
        return;
    };

    let current_pos = transform.translation;
    let current_chunk = (current_pos.xy() / CHUNK_SIZE).as_ivec2();

    let chunks_needed = get_chunks_needed(&current_chunk, RENDER_DISTANCE_X, RENDER_DISTANCE_Y);

    for chunk in &chunks_needed {
        if current_chunks.0.contains(&chunk) {
            continue;
        };

        current_chunks.0.insert(*chunk);

        let chunk_pos = chunk.as_vec2() * CHUNK_SIZE;
        let chunk_x_range = (chunk_pos.x)..(chunk_pos.x + CHUNK_SIZE);
        let chunk_y_range = (chunk_pos.y)..(chunk_pos.y + CHUNK_SIZE);

        let mut rng = thread_rng();
        if rng.gen_bool(ITEM_RATE.into()) && IVec2::ZERO != *chunk {
            let spawn_location = Vec2::new(
                rng.gen_range(chunk_x_range.clone()),
                rng.gen_range(chunk_y_range.clone()),
            );
            let item = item_spawn_table.item_rates[item_weights.sample(&mut rng)].item_type;

            commands.spawn((
                Chunk { pos: *chunk },
                ItemPickup { item_type: item },
                Collider::ball(32.),
                Sensor,
                ActiveEvents::COLLISION_EVENTS,
                SpriteBundle {
                    texture: asset_server.load(get_item_sprite(&item)),
                    transform: Transform::from_translation(spawn_location.extend(0.)),
                    ..default()
                },
                YSort(0.),
            ));
        }
        if rng.gen_bool(ROCK_RATE.into()) && IVec2::ZERO != *chunk {
            let spawn_location = Vec2::new(
                rng.gen_range(chunk_x_range.clone()),
                rng.gen_range(chunk_y_range.clone()),
            );
            commands.spawn((
                Chunk { pos: *chunk },
                EnemyKnockback { knockback: 160. },
                ContactEnemy,
                Collider::ball(16.),
                Sensor,
                ActiveEvents::COLLISION_EVENTS,
                SpriteBundle {
                    texture: asset_server.load("sprites/obstacles/rock.png"),
                    transform: Transform::from_translation(spawn_location.extend(0.)),
                    ..default()
                },
                YSort(0.),
            ));
        }

        commands.spawn((
            Chunk { pos: *chunk },
            SpriteBundle {
                texture: asset_server.load("sprites/tiles/water_layer_2.png"),
                transform: Transform::from_translation(chunk_pos.extend(0.)),
                sprite: Sprite {
                    custom_size: Some(Vec2::splat(CHUNK_SIZE)),
                    color: Color::rgba(1., 1., 1., 0.5),
                    ..default()
                },
                ..default()
            },
            ImageScaleMode::Tiled {
                tile_x: true,
                tile_y: true,
                stretch_value: 1.0, // The image will tile every 128px
            },
            AnimateOffset {
                angle: 0.25 * PI,
                speed: 3.,
            },
            YSort(-1.),
        ));

        commands.spawn((
            Chunk { pos: *chunk },
            SpriteBundle {
                texture: asset_server.load("sprites/tiles/water_layer_1.png"),
                transform: Transform::from_translation(chunk_pos.extend(0.)),
                sprite: Sprite {
                    custom_size: Some(Vec2::splat(CHUNK_SIZE)),
                    ..default()
                },
                ..default()
            },
            ImageScaleMode::Tiled {
                tile_x: true,
                tile_y: true,
                stretch_value: 1.0, // The image will tile every 128px
            },
            AnimateOffset {
                angle: 0.75 * PI,
                speed: 1.3,
            },
            YSort(-2.),
        ));

        commands.spawn((
            Chunk { pos: *chunk },
            SpriteBundle {
                texture: asset_server.load("sprites/tiles/water_bg.png"),
                transform: Transform::from_translation(chunk_pos.extend(0.)),
                sprite: Sprite {
                    custom_size: Some(Vec2::splat(CHUNK_SIZE)),
                    ..default()
                },
                ..default()
            },
            ImageScaleMode::Tiled {
                tile_x: true,
                tile_y: true,
                stretch_value: 1.0, // The image will tile every 128px
            },
            YSort(-3.),
        ));
    }

    // Clear unused chunks
    for (entity, chunk) in chunks_query.iter() {
        if !chunks_needed.contains(&chunk.pos) {
            current_chunks.0.remove(&chunk.pos);
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn cleanup_chunks(
    mut commands: Commands,
    chunks_query: Query<Entity, With<Chunk>>,
    mut current_chunks: ResMut<CurrentChunks>,
) {
    current_chunks.0.clear();

    for entity in chunks_query.iter() {
        commands.entity(entity).despawn();
    }
}

fn update_offset(
    ingame_time: Res<IngameTime>,
    mut offsets_query: Query<(&Chunk, &AnimateOffset, &mut Transform)>,
) {
    for (chunk, animate_offset, mut transform) in offsets_query.iter_mut() {
        let pos = chunk.pos.as_vec2() * CHUNK_SIZE;

        let mut pos_offset =
            Vec2::from_angle(animate_offset.angle) * animate_offset.speed * ingame_time.0;
        pos_offset.x %= TILE_SIZE;
        pos_offset.y %= TILE_SIZE;
        pos_offset = pos_offset.trunc();

        let final_pos = pos + pos_offset;

        transform.translation.x = final_pos.x;
        transform.translation.y = final_pos.y;
    }
}

fn handle_item_pickups(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut inventory: ResMut<Inventory>,
    mut item_pickups_query: Query<(&ItemPickup, Entity)>,
    player_query: Query<&Player>,
) {
    for collision_event in collision_events.read() {
        match collision_event {
            CollisionEvent::Started(entity_one, entity_two, _) => {
                let entities = [entity_one, entity_two];
                let mut maybe_item = item_pickups_query.iter_many_mut(entities);
                let mut maybe_player = player_query.iter_many(entities);

                if let (Some((item_pickup, item_entity)), Some(_player)) =
                    (maybe_item.fetch_next(), maybe_player.fetch_next())
                {
                    commands.entity(item_entity).despawn_recursive();
                    inventory.0.push_back(item_pickup.item_type);
                    if inventory.0.len() > INVENTORY_SIZE {
                        inventory.0.pop_front();
                    }
                };
            }
            _ => {}
        }
    }
}

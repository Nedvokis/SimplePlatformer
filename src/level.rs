use avian2d::prelude::*;
use bevy::prelude::*;
use serde::Deserialize;

use crate::player::{Player, SpawnPoint};
use crate::progress::PlayerProgress;
use crate::states::GameState;

#[derive(Deserialize)]
pub struct LevelData {
    #[allow(dead_code)]
    pub name: String,
    pub spawn: (f32, f32),
    pub exit: (f32, f32),
    pub tiles: Vec<TileEntry>,
}

#[derive(Deserialize)]
pub struct TileEntry {
    pub x: i32,
    pub y: i32,
    pub kind: TileKind,
}

#[derive(Deserialize, Clone, Copy)]
pub enum TileKind {
    Platform,
    Spikes,
}

#[derive(Component)]
pub struct Platform;

#[derive(Component)]
pub struct Spikes;

#[derive(Component)]
pub struct Exit;

#[derive(Resource)]
pub struct CurrentLevel(pub usize);

const TILE_SIZE: f32 = 32.0;

pub const LEVELS: &[&str] = &[
    "assets/levels/level_01.ron",
    "assets/levels/level_02.ron",
    "assets/levels/level_03.ron",
    "assets/levels/level_04.ron",
    "assets/levels/level_05.ron",
];

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CurrentLevel(0))
            .add_systems(OnEnter(GameState::Playing), load_level)
            .add_systems(
                Update,
                (check_exit, check_spikes).run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnEnter(GameState::LevelTransition), transition_to_playing);
    }
}

fn load_level(mut commands: Commands, current_level: Res<CurrentLevel>, mut spawn_point: ResMut<SpawnPoint>) {
    let index = current_level.0;
    let contents = std::fs::read_to_string(LEVELS[index])
        .unwrap_or_else(|e| panic!("Failed to read level file {}: {}", LEVELS[index], e));
    let level: LevelData = ron::from_str(&contents)
        .unwrap_or_else(|e| panic!("Failed to parse level file {}: {}", LEVELS[index], e));

    // Set spawn point
    spawn_point.0 = Vec2::new(level.spawn.0 * TILE_SIZE, level.spawn.1 * TILE_SIZE);

    // Spawn tiles
    for tile in &level.tiles {
        let pos = Vec3::new(tile.x as f32 * TILE_SIZE, tile.y as f32 * TILE_SIZE, 0.0);
        match tile.kind {
            TileKind::Platform => {
                commands.spawn((
                    Platform,
                    Sprite {
                        color: Color::srgb(0.4, 0.4, 0.4),
                        custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                        ..default()
                    },
                    Transform::from_translation(pos),
                    RigidBody::Static,
                    Collider::rectangle(TILE_SIZE, TILE_SIZE),
                    Friction::ZERO,
                    DespawnOnExit::<GameState>(GameState::Playing),
                ));
            }
            TileKind::Spikes => {
                commands.spawn((
                    Spikes,
                    Sprite {
                        color: Color::srgb(0.9, 0.2, 0.2),
                        custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                        ..default()
                    },
                    Transform::from_translation(pos),
                    RigidBody::Static,
                    Collider::rectangle(TILE_SIZE, TILE_SIZE),
                    Sensor,
                    CollidingEntities::default(),
                    DespawnOnExit::<GameState>(GameState::Playing),
                ));
            }
        }
    }

    // Spawn exit
    let exit_pos = Vec3::new(level.exit.0 * TILE_SIZE, level.exit.1 * TILE_SIZE, 0.0);
    commands.spawn((
        Exit,
        Sprite {
            color: Color::srgb(0.2, 0.9, 0.2),
            custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
            ..default()
        },
        Transform::from_translation(exit_pos),
        RigidBody::Static,
        Collider::rectangle(TILE_SIZE, TILE_SIZE),
        Sensor,
        CollidingEntities::default(),
        DespawnOnExit::<GameState>(GameState::Playing),
    ));
}

fn check_exit(
    exit_query: Query<&CollidingEntities, With<Exit>>,
    player_query: Query<(), With<Player>>,
    mut current_level: ResMut<CurrentLevel>,
    mut progress: ResMut<PlayerProgress>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for colliding in &exit_query {
        for &entity in colliding.iter() {
            if player_query.get(entity).is_ok() {
                current_level.0 += 1;
                if current_level.0 > progress.max_unlocked_level {
                    progress.max_unlocked_level = current_level.0;
                    crate::progress::save_progress(&progress);
                }
                if current_level.0 < LEVELS.len() {
                    next_state.set(GameState::LevelTransition);
                } else {
                    next_state.set(GameState::LevelSelect);
                }
                return;
            }
        }
    }
}

fn transition_to_playing(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Playing);
}

fn check_spikes(
    spikes_query: Query<&CollidingEntities, With<Spikes>>,
    player_query: Query<(), With<Player>>,
    mut player_transform_query: Query<(&mut Transform, &mut LinearVelocity), With<Player>>,
    spawn_point: Res<SpawnPoint>,
) {
    for colliding in &spikes_query {
        for &entity in colliding.iter() {
            if player_query.get(entity).is_ok() {
                for (mut transform, mut velocity) in &mut player_transform_query {
                    transform.translation = spawn_point.0.extend(0.0);
                    *velocity = LinearVelocity::ZERO;
                }
                return;
            }
        }
    }
}

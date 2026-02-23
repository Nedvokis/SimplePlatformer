use avian2d::prelude::*;
use bevy::prelude::*;

use crate::states::GameState;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Grounded(pub bool);

#[derive(Component)]
pub struct GroundSensor;

#[derive(Resource, Default)]
pub struct SpawnPoint(pub Vec2);

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpawnPoint>()
            .add_systems(OnEnter(GameState::Playing), spawn_player)
            .add_systems(
                Update,
                (ground_detection, player_movement)
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                player_death.run_if(in_state(GameState::Playing)),
            );
    }
}

fn spawn_player(mut commands: Commands, spawn_point: Res<SpawnPoint>) {
    commands
        .spawn((
            Player,
            Grounded(false),
            Sprite {
                color: Color::srgb(0.2, 0.4, 0.9),
                custom_size: Some(Vec2::new(24.0, 32.0)),
                ..default()
            },
            RigidBody::Dynamic,
            Collider::rectangle(24.0, 32.0),
            LockedAxes::ROTATION_LOCKED,
            LinearVelocity::ZERO,
            Transform::from_translation(spawn_point.0.extend(0.0)),
            DespawnOnExit::<GameState>(GameState::Playing),
        ))
        .with_child((
            GroundSensor,
            Collider::rectangle(20.0, 4.0),
            Transform::from_xyz(0.0, -18.0, 0.0),
            Sensor,
            CollidingEntities::default(),
        ));
}

fn ground_detection(
    sensor_query: Query<(&CollidingEntities, &ChildOf), With<GroundSensor>>,
    mut player_query: Query<&mut Grounded, With<Player>>,
) {
    for (colliding, child_of) in &sensor_query {
        if let Ok(mut grounded) = player_query.get_mut(child_of.parent()) {
            grounded.0 = !colliding.is_empty();
        }
    }
}

fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut LinearVelocity, &Grounded), With<Player>>,
) {
    for (mut velocity, grounded) in &mut query {
        let left = keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft);
        let right = keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight);

        if left && !right {
            velocity.x = -300.0;
        } else if right && !left {
            velocity.x = 300.0;
        } else {
            velocity.x = 0.0;
        }

        if keyboard.just_pressed(KeyCode::Space) && grounded.0 {
            velocity.y = 500.0;
        }
    }
}

fn player_death(
    mut query: Query<(&mut Transform, &mut LinearVelocity), With<Player>>,
    spawn_point: Res<SpawnPoint>,
) {
    for (mut transform, mut velocity) in &mut query {
        if transform.translation.y < -500.0 {
            transform.translation = spawn_point.0.extend(0.0);
            *velocity = LinearVelocity::ZERO;
        }
    }
}

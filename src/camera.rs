use bevy::prelude::*;

use crate::player::Player;
use crate::states::GameState;

const CAMERA_SPEED: f32 = 5.0;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(
                Update,
                camera_follow.run_if(in_state(GameState::Playing)),
            );
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn camera_follow(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
    time: Res<Time>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };

    let player_x = player_transform.translation.x;
    let player_y = player_transform.translation.y;
    let camera_x = camera_transform.translation.x;
    let camera_y = camera_transform.translation.y;

    camera_transform.translation.x = camera_x + (player_x - camera_x) * CAMERA_SPEED * time.delta_secs();
    camera_transform.translation.y = camera_y + (player_y - camera_y) * CAMERA_SPEED * time.delta_secs();
}

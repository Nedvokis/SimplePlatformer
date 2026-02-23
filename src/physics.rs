use avian2d::prelude::*;
use bevy::prelude::*;

use crate::states::GameState;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(avian2d::PhysicsPlugins::default())
            .insert_resource(Gravity(Vec2::new(0.0, -980.0)))
            .add_systems(OnEnter(GameState::Playing), unpause_physics)
            .add_systems(OnExit(GameState::Playing), pause_physics);
    }
}

fn pause_physics(mut physics_time: ResMut<Time<Physics>>) {
    physics_time.pause();
}

fn unpause_physics(mut physics_time: ResMut<Time<Physics>>) {
    physics_time.unpause();
}

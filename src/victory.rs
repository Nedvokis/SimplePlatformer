use bevy::prelude::*;

use crate::player::DeathCounter;
use crate::states::GameState;

#[derive(Component)]
struct VictoryAction;

pub struct VictoryPlugin;

impl Plugin for VictoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Victory), spawn_victory_screen)
            .add_systems(
                Update,
                victory_action.run_if(in_state(GameState::Victory)),
            );
    }
}

fn death_comment(total: usize) -> &'static str {
    match total {
        0 => " - flawless!",
        69 => " - nice",
        42 => " - the answer!",
        100 => " - centurion!",
        _ if total > 200 => " - respect for perseverance!",
        _ => "",
    }
}

fn spawn_victory_screen(mut commands: Commands, counter: Res<DeathCounter>) {
    let total = counter.total;
    let comment = death_comment(total);

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            DespawnOnExit::<GameState>(GameState::Victory),
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("CONGRATULATIONS!"),
                TextFont {
                    font_size: 60.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.8, 0.2)),
                Node {
                    margin: UiRect::bottom(Val::Px(30.0)),
                    ..default()
                },
            ));

            // Death count
            parent.spawn((
                Text::new(format!("You died: {}{}", total, comment)),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
            ));

            // Main Menu button
            parent
                .spawn((
                    Button,
                    VictoryAction,
                    Node {
                        width: Val::Px(300.0),
                        height: Val::Px(50.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.3, 0.3, 0.7)),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("Main Menu"),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

fn victory_action(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    buttons: Query<&Interaction, (With<VictoryAction>, Changed<Interaction>)>,
) {
    let enter = keyboard.just_pressed(KeyCode::Enter);
    let clicked = buttons.iter().any(|i| *i == Interaction::Pressed);

    if enter || clicked {
        commands.remove_resource::<DeathCounter>();
        next_state.set(GameState::Menu);
    }
}

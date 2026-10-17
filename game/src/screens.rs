use bevy::prelude::*;

use crate::level::GameOver;

pub struct ScreensPlugin;

impl Plugin for ScreensPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .add_systems(OnEnter(AppState::Intro), spawn_intro_screen)
            .add_systems(OnExit(AppState::Intro), despawn_screen::<IntroScreen>)
            .add_systems(OnEnter(AppState::GameOver), spawn_game_over_screen)
            .add_systems(
                Update,
                enter_game_over_on_exit_reached.run_if(in_state(AppState::Playing)),
            );
    }
}

/// Drives which screen is shown. `player::PlayerPlugin`'s movement systems
/// only run during `Playing` (see their `run_if`), so the player can't act
/// on the intro or game-over screens.
#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppState {
    #[default]
    Intro,
    Playing,
    GameOver,
}

#[derive(Component)]
struct IntroScreen;

#[derive(Component)]
struct GameOverScreen;

/// Moves to `AppState::GameOver` once `level::GameOver` (set by
/// `player::check_exit_reached` when the player reaches the `Exit` tile)
/// flips to `true`.
fn enter_game_over_on_exit_reached(
    game_over: Res<GameOver>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if game_over.0 {
        next_state.set(AppState::GameOver);
    }
}

fn despawn_screen<T: Component>(mut commands: Commands, screens: Query<Entity, With<T>>) {
    for entity in &screens {
        commands.entity(entity).despawn();
    }
}

fn spawn_intro_screen(mut commands: Commands) {
    commands
        .spawn((
            IntroScreen,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::BLACK),
        ))
        .with_children(|screen| {
            screen
                .spawn((
                    Node {
                        padding: UiRect::px(24.0, 24.0, 12.0, 12.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.25, 0.25, 0.25)),
                ))
                .observe(on_play_clicked)
                .with_children(|button| {
                    button.spawn((
                        Text::from("Play"),
                        TextFont {
                            font_size: FontSize::Px(32.0),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

fn on_play_clicked(_click: On<PointerClick>, mut next_state: ResMut<NextState<AppState>>) {
    next_state.set(AppState::Playing);
}

fn spawn_game_over_screen(mut commands: Commands) {
    commands
        .spawn((
            GameOverScreen,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::BLACK),
        ))
        .with_children(|screen| {
            screen.spawn((
                Text::from("The end"),
                TextFont {
                    font_size: FontSize::Px(48.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

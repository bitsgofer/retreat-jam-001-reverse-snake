use bevy::prelude::*;

mod player;
use crate::player::PlayerPlugin;

mod level;
use crate::level::LevelPlugin;

mod tunables;
use crate::tunables::TunablesPlugin;

mod screens;
use crate::screens::ScreensPlugin;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    // file_path: /* ENV["BEVY_ASSET_ROOT"] */,
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
                .set(bevy::log::LogPlugin {
                    level: bevy::log::Level::INFO,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "reverse-snake".into(),
                        // No-op outside wasm32: resizes the canvas to fill
                        // its parent element, so the itch.io browser embed
                        // isn't stuck at a fixed pixel size.
                        fit_canvas_to_parent: true,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_systems(Startup, setup_camera)
        .add_plugins(TunablesPlugin)
        .add_plugins(ScreensPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(LevelPlugin)
        .run();
}

fn print_copyright() {
    info!("Copyright © 2026 BitsGofer Studio.");
}
fn setup_camera(mut commands: Commands) {
    print_copyright();
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: bevy::camera::ScalingMode::Fixed {
                width: 320.0,
                height: 180.0,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
}

use std::fmt;

use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext};
use bevy::prelude::*;
use serde::Deserialize;

const TUNABLES_PATH: &str = "tunables.ron";

pub struct TunablesPlugin;

impl Plugin for TunablesPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<Tunables>()
            .init_asset_loader::<TunablesLoader>()
            .add_systems(Startup, load_tunables);
    }
}

/// Values designers tweak often, loaded from `assets/tunables.ron`.
/// With the `dev` feature's asset hot-reload, saving that file updates
/// these live, no rebuild needed.
#[derive(Asset, TypePath, Deserialize, Clone, Copy, Debug)]
pub struct Tunables {
    /// Pixels per second the player moves across a grid step.
    pub move_speed: f32,
    /// Seconds each animation frame is held for.
    pub seconds_per_frame: f32,
}

impl Default for Tunables {
    fn default() -> Self {
        Tunables {
            move_speed: 50.0,
            seconds_per_frame: 0.1,
        }
    }
}

#[derive(Resource)]
pub struct TunablesHandle(pub Handle<Tunables>);

fn load_tunables(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(TunablesHandle(asset_server.load(TUNABLES_PATH)));
}

/// Reads the current tunables, falling back to defaults while
/// `tunables.ron` is still loading.
pub fn current(handle: &TunablesHandle, assets: &Assets<Tunables>) -> Tunables {
    assets.get(&handle.0).copied().unwrap_or_default()
}

#[derive(Default, TypePath)]
struct TunablesLoader;

#[non_exhaustive]
#[derive(Debug)]
enum TunablesLoaderError {
    Io(std::io::Error),
    Ron(ron::error::SpannedError),
}

impl fmt::Display for TunablesLoaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TunablesLoaderError::Io(e) => write!(f, "could not read tunables file: {e}"),
            TunablesLoaderError::Ron(e) => write!(f, "could not parse tunables RON: {e}"),
        }
    }
}

impl std::error::Error for TunablesLoaderError {}

impl From<std::io::Error> for TunablesLoaderError {
    fn from(e: std::io::Error) -> Self {
        TunablesLoaderError::Io(e)
    }
}

impl From<ron::error::SpannedError> for TunablesLoaderError {
    fn from(e: ron::error::SpannedError) -> Self {
        TunablesLoaderError::Ron(e)
    }
}

impl AssetLoader for TunablesLoader {
    type Asset = Tunables;
    type Settings = ();
    type Error = TunablesLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        Ok(ron::de::from_bytes::<Tunables>(&bytes)?)
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

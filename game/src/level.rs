use std::collections::HashMap;
use std::fmt;

use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext};
use bevy::prelude::*;
use serde::Deserialize;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<LevelLayout>()
            .init_asset_loader::<LevelLayoutLoader>()
            .init_resource::<GameOver>()
            .add_systems(
                Startup,
                (load_level_layout, setup, setup_level, spawn_tiles).chain(),
            )
            .add_systems(Update, rebuild_level_on_layout_change);
    }
}

/// Set once the player reaches the `Exit` tile; see
/// `player::check_exit_reached` (sets it) and `player::start_grid_step`
/// (reads it, to stop the player moving once the game has ended).
#[derive(Resource, Default)]
pub struct GameOver(pub bool);

pub const TILE_SIZE: f32 = 16.0;

const LEVEL_LAYOUT_PATH: &str = "level_layout.ron";

/// Hand-placed wall tiles added on top of the grid's auto-generated
/// border, loaded from `assets/level_layout.ron`. With the `dev`
/// feature's asset hot-reload, saving that file updates this on the
/// next run — no rebuild needed.
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct LevelLayout {
    pub walls: Vec<(i32, i32)>, // extra (row, col) wall tiles
}

impl Default for LevelLayout {
    fn default() -> Self {
        LevelLayout {
            walls: (-3..=2).map(|row| (row, -4)).collect(),
        }
    }
}

#[derive(Resource)]
struct LevelLayoutHandle(Handle<LevelLayout>);

fn load_level_layout(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(LevelLayoutHandle(asset_server.load(LEVEL_LAYOUT_PATH)));
}

fn current_layout(handle: &LevelLayoutHandle, assets: &Assets<LevelLayout>) -> LevelLayout {
    assets.get(&handle.0).cloned().unwrap_or_default()
}

#[derive(Default, TypePath)]
struct LevelLayoutLoader;

#[non_exhaustive]
#[derive(Debug)]
enum LevelLayoutLoaderError {
    Io(std::io::Error),
    Ron(ron::error::SpannedError),
}

impl fmt::Display for LevelLayoutLoaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LevelLayoutLoaderError::Io(e) => write!(f, "could not read level layout file: {e}"),
            LevelLayoutLoaderError::Ron(e) => write!(f, "could not parse level layout RON: {e}"),
        }
    }
}

impl std::error::Error for LevelLayoutLoaderError {}

impl From<std::io::Error> for LevelLayoutLoaderError {
    fn from(e: std::io::Error) -> Self {
        LevelLayoutLoaderError::Io(e)
    }
}

impl From<ron::error::SpannedError> for LevelLayoutLoaderError {
    fn from(e: ron::error::SpannedError) -> Self {
        LevelLayoutLoaderError::Ron(e)
    }
}

impl AssetLoader for LevelLayoutLoader {
    type Asset = LevelLayout;
    type Settings = ();
    type Error = LevelLayoutLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        Ok(ron::de::from_bytes::<LevelLayout>(&bytes)?)
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TileKind {
    Floor,
    Wall,
    Exit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SpriteSheetKind {
    Level,
}

#[derive(Resource, Debug)]
struct SpriteSheetRegistry {
    spritesheets: HashMap<SpriteSheetKind, (Handle<Image>, Handle<TextureAtlasLayout>)>,
}

#[derive(Clone, Copy, Debug)]
pub struct Tile {
    pub kind: TileKind,
    pub spritesheet: SpriteSheetKind,
    pub index: usize, // index insdie the spritesheet
    pub width: u32,   // # of tiles
    pub height: u32,  // # of tiles
}

impl Tile {
    fn floor() -> Self {
        Tile {
            kind: TileKind::Floor,
            spritesheet: SpriteSheetKind::Level,
            index: 0,
            width: 1,
            height: 1,
        }
    }

    fn wall() -> Self {
        Tile {
            kind: TileKind::Wall,
            spritesheet: SpriteSheetKind::Level,
            index: 1,
            width: 1,
            height: 1,
        }
    }

    fn exit() -> Self {
        Tile {
            kind: TileKind::Exit,
            spritesheet: SpriteSheetKind::Level,
            index: 2,
            width: 1,
            height: 1,
        }
    }

    fn width_px(&self) -> f32 {
        self.width as f32 * TILE_SIZE
    }

    fn height_px(&self) -> f32 {
        self.height as f32 * TILE_SIZE
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GridPos {
    pub row: i32, // vertical index, used for Y
    pub col: i32, // horizontal index, used for X
}

/// The (row, col) location of a cell inside a grid/matrix.
impl GridPos {
    pub fn new(row: i32, col: i32) -> Self {
        GridPos { row, col }
    }

    /// Translate the original GridPos to a new coordinate.
    pub fn translate(self, delta_row: i32, delta_col: i32) -> GridPos {
        GridPos::new(self.row + delta_row, self.col + delta_col)
    }

    /// Return the (x, y) world coordinates of the cell's bottom-left anchor.
    pub fn to_world(self) -> Vec2 {
        // IMPORTANT! row & col are swapped when making coordinates.
        // We tend to think of cells via (row, col): vertical then horizontal.
        // But we tend to think of coordinates as (x, y): horizontal then vertical.
        Vec2::new(self.col as f32 * TILE_SIZE, self.row as f32 * TILE_SIZE)
    }
}

/// Marks an entity spawned by [`spawn_tile_sprites`], so a level rebuild
/// can find and despawn the previous tiles before respawning new ones.
#[derive(Component)]
struct TileSprite;

#[derive(Resource)]
pub struct TileGrid {
    pub tiles: HashMap<GridPos, Tile>, // (row, col)
    pub width: i32,
    pub height: i32,
}

impl TileGrid {
    fn new(width: i32, height: i32) -> Self {
        TileGrid {
            tiles: HashMap::new(),
            width,
            height,
        }
    }

    fn col_range(&self) -> std::ops::RangeInclusive<i32> {
        // -(self.width / 2)..(self.width - self.width / 2)
        self.left()..=self.right()
    }

    fn row_range(&self) -> std::ops::RangeInclusive<i32> {
        // -(self.height / 2)..(self.height - self.height / 2)
        self.bottom()..=self.top()
    }

    fn bottom(&self) -> i32 {
        -(self.height / 2)
    }
    fn top(&self) -> i32 {
        (self.height - self.height / 2) - 1
    } // inclusive
    fn left(&self) -> i32 {
        -(self.width / 2)
    }
    fn right(&self) -> i32 {
        (self.width - (self.width / 2)) - 1
    } // inclusive

    fn set_tile(&mut self, pos: GridPos, tile: Tile) {
        if self.col_range().contains(&pos.col) && self.row_range().contains(&pos.row) {
            self.tiles.insert(pos, tile);
        }
    }

    pub fn get_tile(&self, pos: GridPos) -> Option<Tile> {
        self.tiles.get(&pos).copied()
    }

    pub fn is_walkable(&self, pos: GridPos) -> bool {
        matches!(
            self.get_tile(pos).map(|tile| tile.kind),
            Some(TileKind::Floor) | Some(TileKind::Exit)
        )
    }

    fn grid_to_world(&self, row: i32, col: i32) -> Vec2 {
        Vec2::new(col as f32 * TILE_SIZE, row as f32 * TILE_SIZE)
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let mut spritesheet_registry = SpriteSheetRegistry {
        spritesheets: HashMap::new(),
    };

    let img_level = asset_server.load("level.png");
    let mut layout_level = TextureAtlasLayout::new_empty(UVec2::new(64, 64));

    layout_level.add_texture(bevy::math::URect {
        min: UVec2::new(0, 0),
        max: UVec2::new(16, 16),
    }); // index 0
    layout_level.add_texture(bevy::math::URect {
        min: UVec2::new(16, 0),
        max: UVec2::new(32, 16),
    }); // index 1
    layout_level.add_texture(bevy::math::URect {
        min: UVec2::new(32, 0),
        max: UVec2::new(48, 16),
    }); // index 2

    let layout_handle_level = atlas_layouts.add(layout_level);
    spritesheet_registry
        .spritesheets
        .insert(SpriteSheetKind::Level, (img_level, layout_handle_level));
    commands.insert_resource(spritesheet_registry);
}

/// Builds the grid: floor everywhere, an auto-generated border (derived
/// from the grid's own dimensions), plus the hand-placed walls from
/// `layout`.
fn build_tile_grid(layout: &LevelLayout) -> TileGrid {
    // let mut tile_grid = TileGrid::new(20, 12);
    let mut tile_grid = TileGrid::new(20, 10);

    for row in tile_grid.row_range() {
        for col in tile_grid.col_range() {
            tile_grid.set_tile(GridPos::new(row, col), Tile::floor());
        }
    }

    // border walls, generated from the grid's own dimensions
    let bottom = tile_grid.bottom();
    let top = tile_grid.top();
    let left = tile_grid.left();
    let right = tile_grid.right();
    for col in left..=right {
        tile_grid.set_tile(GridPos::new(top, col), Tile::wall());
        tile_grid.set_tile(GridPos::new(bottom, col), Tile::wall());
    }
    for row in bottom..=top {
        tile_grid.set_tile(GridPos::new(row, left), Tile::wall());
        tile_grid.set_tile(GridPos::new(row, right), Tile::wall());
    }

    // hand-placed walls, editable via assets/level_layout.ron
    for (row, col) in &layout.walls {
        tile_grid.set_tile(GridPos::new(*row, *col), Tile::wall());
    }

    // the maze's exit; reaching it ends the game (see player::check_exit_reached)
    tile_grid.set_tile(GridPos::new(-4, -9), Tile::exit());

    tile_grid
}

fn setup_level(
    mut commands: Commands,
    layout_handle: Res<LevelLayoutHandle>,
    layout_assets: Res<Assets<LevelLayout>>,
) {
    let layout = current_layout(&layout_handle, &layout_assets);
    commands.insert_resource(build_tile_grid(&layout));
}

/// Spawns one sprite per tile in `tile_grid`, marked with [`TileSprite`]
/// so a later rebuild can find and despawn them.
fn spawn_tile_sprites(
    commands: &mut Commands,
    tile_grid: &TileGrid,
    spritesheet_registry: &SpriteSheetRegistry,
) {
    for (pos, tile) in &tile_grid.tiles {
        // let world_pos = tile_grid.grid_to_world(*row, *col);
        let world_pos = pos.to_world();

        let Some((handle_image, handle_texture_atlas_layout)) =
            spritesheet_registry.spritesheets.get(&tile.spritesheet)
        else {
            continue;
        };

        commands.spawn((
            Sprite::from_atlas_image(
                // atlas_handle.image.clone(),
                handle_image.clone(),
                TextureAtlas {
                    // layout: atlas_handle.layout.clone(),
                    layout: handle_texture_atlas_layout.clone(),
                    index: tile.index,
                },
            ),
            Transform::from_translation(world_pos.extend(0.0)),
            bevy::sprite::Anchor::BOTTOM_LEFT,
            Name::new(format!("Tile({}, {})", pos.row, pos.col)),
            TileSprite,
        ));
    }
}

fn spawn_tiles(
    mut commands: Commands,
    tile_grid: Res<TileGrid>,
    spritesheet_registry: Res<SpriteSheetRegistry>,
) {
    spawn_tile_sprites(&mut commands, &tile_grid, &spritesheet_registry);
}

/// Reacts to `assets/level_layout.ron` finishing its (async) initial load
/// or being hot-reloaded while the game is running: despawns the current
/// tile sprites and rebuilds the grid from the fresh layout.
///
/// `setup_level`/`spawn_tiles` build an initial grid at `Startup` using
/// whatever `current_layout` returns (likely `LevelLayout::default()`,
/// since the real file is still loading asynchronously at that point) so
/// `TileGrid` always exists before any `Update` system — such as
/// `player::start_grid_step` — looks for it. This system is what corrects
/// that initial guess once the real file loads, and keeps it in sync on
/// every subsequent edit.
fn rebuild_level_on_layout_change(
    mut commands: Commands,
    mut events: MessageReader<AssetEvent<LevelLayout>>,
    layout_handle: Res<LevelLayoutHandle>,
    layout_assets: Res<Assets<LevelLayout>>,
    spritesheet_registry: Res<SpriteSheetRegistry>,
    existing_tiles: Query<Entity, With<TileSprite>>,
) {
    let id = layout_handle.0.id();
    let should_rebuild = events.read().any(|event| {
        event.is_added(id) || event.is_modified(id) || event.is_loaded_with_dependencies(id)
    });
    if !should_rebuild {
        return;
    }

    info!("Rebuilding level");

    let Some(layout) = layout_assets.get(&layout_handle.0) else {
        return;
    };

    for entity in &existing_tiles {
        commands.entity(entity).despawn();
    }

    let tile_grid = build_tile_grid(layout);
    spawn_tile_sprites(&mut commands, &tile_grid, &spritesheet_registry);
    commands.insert_resource(tile_grid);
}

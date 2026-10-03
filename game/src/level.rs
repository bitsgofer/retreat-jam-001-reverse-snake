use bevy::prelude::*;
use std::collections::HashMap;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Startup, (setup, setup_level, spawn_tiles).chain());
	}
}

const TILE_SIZE: f32 = 16.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TileKind {
	Floor,
	Wall,
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

	fn width_px(&self) -> f32 {
		self.width as f32 * TILE_SIZE
	}

	fn height_px(&self) -> f32 {
		self.height as f32 * TILE_SIZE
	}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GridPos {
    pub row: i32, // vertical index, used for Y
    pub col: i32, // horizontal index, used for X
}

impl GridPos {
    pub fn new(row: i32, col: i32) -> Self {
        GridPos{row, col}
    }

    pub fn to_world(self) -> Vec2 {
        // IMPORTANT! row & col are swapped when making coordinates.
        // We tend to think of cells via (row, col): vertical then horizontal.
        // But we tend to think of coordinates as (x, y): horizontal then vertical.
		Vec2::new(self.col as f32 * TILE_SIZE, self.row as f32 * TILE_SIZE)

    }
}

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

    fn bottom(&self) -> i32 { -(self.height/2) }
    fn top(&self) -> i32 { (self.height - self.height/2)-1 } // inclusive
    fn left(&self) -> i32 { -(self.width/2) }
    fn right(&self) -> i32 { (self.width-(self.width/2)) - 1 } // inclusive

	fn set_tile(&mut self, pos: GridPos, tile: Tile) {
		if self.col_range().contains(&pos.col) && self.row_range().contains(&pos.row) {
			self.tiles.insert(pos, tile);
		}
	}

	fn get_tile(&self, pos: GridPos) -> Option<Tile> {
		self.tiles.get(&pos).copied()
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

	let layout_handle_level = atlas_layouts.add(layout_level);
	spritesheet_registry.spritesheets.insert(
		SpriteSheetKind::Level,
		(img_level, layout_handle_level),
	);
	commands.insert_resource(spritesheet_registry);
}

fn setup_level(mut commands: Commands) {
	// let mut tile_grid = TileGrid::new(20, 12);
	let mut tile_grid = TileGrid::new(20, 10);

	for row in tile_grid.row_range() {
		for col in tile_grid.col_range() {
			tile_grid.set_tile(GridPos::new(row, col), Tile::floor());
		}
	}

    // walls
	// let bottom = tile_grid.row_range().start;
	// let top = tile_grid.row_range().end-1;
	// let left = tile_grid.col_range().start;
	// let right = tile_grid.col_range().end-1;
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

	commands.insert_resource(tile_grid);
}

fn spawn_tiles(
	mut commands: Commands,
	tile_grid: Res<TileGrid>,
	spritesheet_registry: Res<SpriteSheetRegistry>,
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
        ));
	}
}

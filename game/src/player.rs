use bevy::prelude::*;

use crate::level::GridPos;
use crate::level::TileGrid;
use crate::level::TILE_SIZE as GRID_TILE_SIZE;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player).add_systems(
            Update,
            (tween_grid_step, start_grid_step, animate_player).chain(),
        );
    }
}

#[derive(Component)]
struct Player;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
enum Facing {
    Up,
    Left,
    Down,
    Right,
}

// Character-specific spritesheet
const SPRITE_PATH: &str = "player.png";
const TILE_SIZE: UVec2 = UVec2::new(16, 16); // (width, height) of one sprite
const MAX_FRAMES_PER_ANIMATION: usize = 16;
const ANIMATION_COUNT: usize = 3;
const MOVE_SPEED: f32 = 140.0; // pixels per second; how fast things move in the grid
const SECONDS_PER_FRAME: f32 = 0.1;

#[derive(Component)]
struct AnimationState {
    facing: Facing,
    moving: bool,
    was_moving: bool,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

/// Help tween world coordinates.
#[derive(Component)]
struct TweenCoordinates {
    from: Vec2, // (x, y) world coordinates
    to: Vec2, // (x, y) world coordinates
    timer: Timer,
}

impl TweenCoordinates {
    pub fn new(from: Vec2, to: Vec2, px_per_second: f32) -> Self {
        TweenCoordinates{
            from: from,
            to: to,
            timer: Timer::from_seconds(GRID_TILE_SIZE/px_per_second, TimerMode::Once),
        }
    }
}

fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture = asset_server.load(SPRITE_PATH);
    let layout = atlas_layouts.add(TextureAtlasLayout::from_grid(
        TILE_SIZE,
        MAX_FRAMES_PER_ANIMATION as u32,
        ANIMATION_COUNT as u32,
        None,
        None,
    ));

    commands.spawn((
        Sprite::from_atlas_image(
            // use the very first sprite
            texture,
            TextureAtlas { layout, index: 0 },
        ),
        Transform::from_translation(GridPos::new(0, 0).to_world().extend(0.0)),
        bevy::sprite::Anchor::BOTTOM_LEFT,
        Player,
        GridPos::new(0, 0),
        AnimationState {
            facing: Facing::Down,
            moving: false,
            was_moving: false,
        },
        AnimationTimer(Timer::from_seconds(SECONDS_PER_FRAME, TimerMode::Repeating)),
    ));
}

/// Returns the fixed-priority held direction (Up > Down > Left > Right) as
/// `(d_row, d_col, Facing)`, or `None` if no arrow key is held.
fn pressed_direction(input: &ButtonInput<KeyCode>) -> Option<(i32, i32, Facing)> {
    if input.pressed(KeyCode::ArrowUp) {
        Some((1, 0, Facing::Up))
    } else if input.pressed(KeyCode::ArrowDown) {
        Some((-1, 0, Facing::Down))
    } else if input.pressed(KeyCode::ArrowLeft) {
        Some((0, -1, Facing::Left))
    } else if input.pressed(KeyCode::ArrowRight) {
        Some((0, 1, Facing::Right))
    } else {
        None
    }
}

/// Starts a new grid step for the player if a direction is held and the
/// destination tile is walkable. Runs only when the player has no in-flight
/// `TweenCoordinates`, so a held key chains into the next tile as soon as the
/// current step finishes.
fn start_grid_step(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    tile_grid: Res<TileGrid>,
    mut player: Query<
        (
            Entity,
            &Transform,
            &mut GridPos,
            &mut AnimationState,
            &mut Sprite,
        ),
        (With<Player>, Without<TweenCoordinates>),
    >,
) {
    let Ok((entity, transform, mut pos, mut anim, mut sprite)) = player.single_mut() else {
        return;
    };

    let Some((d_row, d_col, facing)) = pressed_direction(&input) else {
        anim.moving = false;
        return;
    };

    anim.facing = facing;
    // Flip the texture's UVs, not the Transform scale: the sprite is
    // anchored BOTTOM_LEFT (to match tile anchoring), and a negative
    // Transform.scale.x mirrors the quad around that anchor point, making it
    // extend into the *previous* tile instead of the current one.
    sprite.flip_x = facing == Facing::Left;

    let dest = pos.translate(d_row, d_col);
    if !tile_grid.is_walkable(dest) {
        // Blocked by a wall (or the edge of the grid): face the direction
        // but don't move.
        anim.moving = false;
        return;
    }

    commands.entity(entity).insert(TweenCoordinates::new(
        transform.translation.xy(),
        dest.to_world(),
        50.0,
    ));
    *pos = dest;
    anim.moving = true;
}

/// Advances the in-flight `TweenCoordinates` tween, snapping to the destination and
/// removing the component once the step completes.
fn tween_grid_step(
    mut commands: Commands,
    time: Res<Time>,
    mut player: Query<(Entity, &mut Transform, &mut TweenCoordinates, &mut AnimationState), With<Player>>,
) {
    let Ok((entity, mut transform, mut transformTween, mut anim)) = player.single_mut() else {
        return;
    };

    transformTween.timer.tick(time.delta());

    let interpolated = transformTween.from.lerp(transformTween.to, transformTween.timer.fraction());
    transform.translation.x = interpolated.x.round();
    transform.translation.y = interpolated.y.round();

    if transformTween.timer.is_finished() {
        transform.translation.x = transformTween.to.x;
        transform.translation.y = transformTween.to.y;
        commands.entity(entity).remove::<TweenCoordinates>();
        anim.moving = false;
    }
}

fn animate_player(
    time: Res<Time>,
    mut query: Query<(&mut AnimationState, &mut AnimationTimer, &mut Sprite), With<Player>>,
) {
    let Ok((mut anim, mut timer, mut sprite)) = query.single_mut() else {
        return;
    };

    let atlas = match sprite.texture_atlas.as_mut() {
        Some(a) => a,
        None => return,
    };

    // Compute the target row and current position in the atlas
    let target_row = row_zero_based(anim.facing);
    let mut current_row = atlas.index / MAX_FRAMES_PER_ANIMATION;
    let mut current_col = atlas.index % MAX_FRAMES_PER_ANIMATION;

    // If the facing changed (or we weren't on a walking row), snap to the first frame of the target row
    if current_row != target_row {
        atlas.index = row_start_index(anim.facing);
        current_col = 0;
        current_row = target_row;
        timer.reset();
    }

    let just_started = anim.moving && !anim.was_moving;
    let just_stopped = !anim.moving && anim.was_moving;

    if anim.moving {
        if just_started {
            // On tap or movement start, immediately advance one frame for visible feedback
            let row_start = row_start_index(anim.facing);
            let next_col = (current_col + 1) % MAX_FRAMES_PER_ANIMATION;
            atlas.index = row_start + next_col;
            // Restart the timer so the next advance uses a full interval
            timer.reset();
        } else {
            // Continuous movement: advance based on timer cadence
            timer.tick(time.delta());
            if timer.just_finished() {
                let row_start = row_start_index(anim.facing);
                let next_col = (current_col + 1) % MAX_FRAMES_PER_ANIMATION;
                atlas.index = row_start + next_col;
            }
        }
    } else if just_stopped {
        // Not moving: keep current frame to avoid snap. Reset timer on transition to idle.
        timer.reset();
    }

    // Update previous movement state
    anim.was_moving = anim.moving;
}

fn row_start_index(facing: Facing) -> usize {
    row_zero_based(facing) * MAX_FRAMES_PER_ANIMATION
}

fn row_zero_based(facing: Facing) -> usize {
    match facing {
        Facing::Down => 0,
        Facing::Left => 1,
        Facing::Right => 1,
        Facing::Up => 2,
    }
}

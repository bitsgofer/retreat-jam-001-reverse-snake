# Grid-based player movement + wall collision

## Context

The player currently moves by directly mutating `Transform.translation` every
frame (`game/src/player.rs::move_player`, lines 72-124): `direction.normalize()
* MOVE_SPEED * delta`, 8-directional (diagonals allowed), with zero collision
checks — the player walks straight through walls. Meanwhile `game/src/level.rs`
already has a full `TileGrid`/`GridPos` grid with `TileKind::Wall` tiles placed
around the level perimeter (`setup_level`), but nothing reads that data during
movement.

The ask: make the player stop at walls, and make movement itself Pokemon-style
— one discrete tile per step, 4-directional only, with a short animated glide
between tiles instead of either an instant teleport or continuous analog
sliding. The existing hand-rolled sprite-frame animator
(`AnimationState`/`AnimationTimer`/`animate_player`) can be reused unchanged;
it already just reacts to a `moving: bool` + `facing: Facing` state, with no
knowledge of what drives them.

There is no physics engine in this project (no Rapier/Avian — confirmed via
grep), so collision is implemented as a plain `HashMap` lookup against
`TileGrid`, and the "glide" is a small hand-rolled lerp, consistent with the
project's existing "hand-rolled over engine machinery" style.

## Design decisions

1. **4-directional only, fixed key priority** Up > Down > Left > Right when
   multiple arrow keys are held — no diagonal movement, no extra input-history
   state needed.
2. **Held key chains into the next tile automatically** once the current step
   finishes (no re-press needed per tile) — achieved for free by gating the
   "start a step" system on `Without<GridMover>`.
3. **Step duration** `STEP_DURATION_SECS = level::TILE_SIZE / MOVE_SPEED` (≈0.114s),
   derived from the existing `MOVE_SPEED` constant so the jam's existing pacing
   tuning carries over instead of inventing a new number.
4. **Collision**: before starting a step, look up the destination `GridPos` in
   `TileGrid`. Block the step if the tile is `TileKind::Wall` **or** absent
   (`None`, i.e. outside the grid — fail closed). On block: the sprite still
   turns to face that direction, but `GridPos`/`Transform`/`moving` don't change
   (hard stop, no sliding).
5. **`AnimationState.moving`** is reinterpreted as "a `GridMover` tween is
   currently in flight" — `animate_player` needs **zero changes**.

## Files to change

### `game/src/level.rs` (visibility-only changes, no logic changes)
- `const TILE_SIZE: f32 = 16.0;` → `pub const TILE_SIZE: f32 = 16.0;` (line 12)
- `GridPos` derive: add `Component` so it can be attached to the player entity
  (line 69) — it's already `pub`.
- `fn to_world(self) -> Vec2` → `pub fn to_world(...)` (line 80)
- `fn get_tile(&self, pos: GridPos) -> Option<Tile>` → `pub fn get_tile(...)` (line 126)

### `game/src/player.rs` (primary work)
- Import `crate::level::{self, GridPos, TileGrid, TileKind};`
- Add `const STEP_DURATION_SECS: f32 = level::TILE_SIZE / MOVE_SPEED;`
- Add component:
  ```rust
  #[derive(Component)]
  struct GridMover { from: Vec2, to: Vec2, timer: Timer } // TimerMode::Once
  ```
- Add `fn pressed_direction(input: &ButtonInput<KeyCode>) -> Option<(i32, i32, Facing)>`
  returning `(d_row, d_col, Facing)` in fixed priority order (Up/Down/Left/Right)
  — matches `GridPos`'s `(row, col)` field order directly, avoiding an x/y vs
  row/col mixup.
- `spawn_player`: add a `GridPos` component (start at `GridPos::new(0, 0)` —
  verified interior floor tile, see Verification below) and fix the anchor
  mismatch: spawn `Transform` at `spawn_pos.to_world() + Vec2::splat(level::TILE_SIZE / 2.0)`
  instead of `Vec3::ZERO`. (`GridPos::to_world()` returns a tile's bottom-left
  corner since tiles use `Anchor::BOTTOM_LEFT`; the player sprite uses the
  default center anchor, so without this offset the player sits on the corner
  shared by 4 tiles instead of centered in one.)
- Delete `move_player`; replace with two systems:
  - **`start_grid_step`** — query `(Entity, &mut Transform, &mut GridPos, &mut AnimationState), (With<Player>, Without<GridMover>)`. Reads input via `pressed_direction`; if none pressed, set `anim.moving = false` and return. Otherwise set `anim.facing`/`transform.scale.x` flip, compute `dest = GridPos::new(grid_pos.row + d_row, grid_pos.col + d_col)`, check `tile_grid.get_tile(dest)` (block on `Wall` or `None`). If blocked, set `anim.moving = false` and return (no movement). Otherwise `commands.entity(entity).insert(GridMover { from: current_translation, to: dest.to_world() + TILE_SIZE/2 offset, timer: Timer::from_seconds(STEP_DURATION_SECS, TimerMode::Once) })`, commit `*grid_pos = dest` immediately, set `anim.moving = true`.
  - **`tween_grid_step`** — query `(Entity, &mut Transform, &mut GridMover, &mut AnimationState), With<Player>`. Ticks the timer, lerps `transform.translation` from `mover.from` to `mover.to` by progress `t`, rounds to nearest pixel each frame (preserves existing pixel-snap behavior under `ImagePlugin::default_nearest()`). On `timer.finished()`: snap exactly to `to`, `commands.entity(entity).remove::<GridMover>()`, set `anim.moving = false`.
- `animate_player`: **no changes**.
- `PlayerPlugin::build`:
  ```rust
  app.add_systems(Startup, spawn_player)
      .add_systems(Update, (tween_grid_step, start_grid_step, animate_player).chain());
  ```
  Order matters: `tween_grid_step` runs first so a tween that finishes this
  frame clears `GridMover` before `start_grid_step` checks for it, allowing a
  held key to chain into the next tile without a stall; `animate_player` runs
  last so it sees this frame's final `moving` state.

No changes needed to `main.rs`.

## Risks

- **Visible one-time spawn shift**: player now spawns at world `(8, 8)`
  instead of `(0, 0)` — intentional (anchor fix), but worth knowing it's not a
  regression.
- **No camera-follow exists** (pre-existing, unrelated to this change): the
  camera is static at world origin, so walking far from spawn exits the
  viewport in both old and new code.
- **Animation pacing**: `STEP_DURATION_SECS` (~0.114s) is close to
  `SECONDS_PER_FRAME` (0.1s), so each step may show only 1-2 animation frames,
  reading choppier than the old continuous movement. Tune `SECONDS_PER_FRAME`
  or `STEP_DURATION_SECS` after seeing it run if needed.
- **Same-frame chaining relies on system order**: if ordering between
  `tween_grid_step` and `start_grid_step` is ever loosened, held-key chaining
  degrades to a harmless one-frame stall per tile, not a bug.

## Verification

Spawn-position sanity check (grid is `TileGrid::new(20, 10)`): `left()=-10,
right()=9` (walls), `bottom()=-5, top()=4` (walls) → `GridPos::new(0, 0)` is
within floor range `rows -4..=3, cols -9..=8`. Confirmed valid floor spawn.

Manual test via `just run`:
1. Player sprite appears centered inside its spawn tile (not straddling a tile
   corner) — confirms the anchor fix.
2. Walk into each of the 4 perimeter walls: player stops flush, no visual
   clipping/overlap, repeated presses into the wall do nothing further.
3. Tap a direction once: exactly one tile of smooth glide (~0.11s), not an
   instant jump, then a full stop.
4. Hold two perpendicular arrow keys (e.g. Up+Left): movement is only ever in
   one cardinal direction per the fixed priority order, never diagonal.
5. Hold one direction continuously: player glides tile-to-tile with no
   stutter/pause between steps.
6. Walk in all 4 directions: sprite row/flip (`scale.x = -1` for Left) still
   renders correctly, including mid-glide.
7. Run `just lint` (fmt + clippy) to catch dead-code/visibility warnings from
   removing `move_player` and touching visibility (e.g. ensure `MOVE_SPEED` is
   still referenced via `STEP_DURATION_SECS`, not left unused).

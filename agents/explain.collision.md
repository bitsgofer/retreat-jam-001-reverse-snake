# Grid-based player movement + wall collision — summary

This implements the plan in `agents/plan.collision.md`: replace the old
continuous/analog player movement with discrete, Pokemon-style grid steps
that stop at walls, animated with a short glide between tiles.

## `game/src/level.rs` — visibility only, no logic changes

- `TILE_SIZE` made `pub` so `player.rs` can derive its step duration from it.
- `GridPos` gained `#[derive(Component)]` so it can be attached to the player
  entity and queried alongside `Transform`.
- `GridPos::offset(drow, dcol) -> GridPos` — small helper used to compute a
  candidate destination tile from the player's current position and a
  direction.
- `TileGrid::is_walkable(pos) -> bool` — returns `true` only if `pos` has a
  tile *and* that tile's kind is `Floor`. This is the single collision check:
  a `Wall` tile and a missing tile (outside the grid) both read as "not
  walkable," so the grid fails closed at its edges without a separate bounds
  check.

## `game/src/player.rs` — the movement rewrite

### Before

`move_player` ran every frame: it read the arrow keys, built a `Vec2`
direction (normalizing diagonals), and directly added
`direction * MOVE_SPEED * delta_secs()` to `transform.translation`. Movement
was continuous/analog, 8-directional, and had no concept of the tile grid —
the player could walk straight through walls.

### After

Movement is now driven by a `GridPos` component (the player's logical tile)
plus an optional `GridMover` component (present only while a step is
animating):

```rust
struct GridMover { from: Vec2, to: Vec2, timer: Timer } // TimerMode::Once
```

Two systems replace `move_player`, chained so a tween that finishes this
frame is cleared before the next step is considered:

```rust
app.add_systems(Update, (tween_grid_step, start_grid_step, animate_player).chain());
```

- **`start_grid_step`** — query filter `(With<Player>, Without<GridMover>)`,
  i.e. it only runs when the player isn't already mid-step. It reads input
  via `pressed_direction` (see below), updates facing, and checks
  `tile_grid.is_walkable(dest)` for the one tile in that direction. If
  walkable: inserts a `GridMover` tweening from the current pixel position to
  the destination tile's world position over `STEP_DURATION_SECS`, and
  commits `GridPos` to the destination *immediately* (so collision/logic
  state updates in lockstep with the key press, even though the sprite is
  still visually gliding). If blocked, or no key is held: `anim.moving` is
  set to `false` and nothing else happens — the sprite can still turn to
  face a wall without moving into it.

- **`tween_grid_step`** — runs whenever a `GridMover` exists. Ticks the
  timer, lerps the pixel position between `from` and `to` each frame
  (rounded, to preserve the existing pixel-snapped look), and on
  `timer.is_finished()` snaps exactly to `to`, removes the `GridMover`, and
  sets `anim.moving = false`.

  Because `tween_grid_step` runs *before* `start_grid_step` in the chain, a
  step that finishes this frame frees up the `Without<GridMover>` filter in
  time for `start_grid_step` to immediately start the next one if the key is
  still held — so holding a direction glides tile-to-tile with no stutter,
  without any extra "was a key held last frame" bookkeeping.

- **`pressed_direction`** — new helper, fixed priority **Up > Down > Left >
  Right**. Only one direction is ever returned, so movement is strictly
  4-directional (no diagonals, unlike the old `Vec2`-based code).

- **`STEP_DURATION_SECS = TILE_SIZE / MOVE_SPEED`** — derived from the
  existing `MOVE_SPEED` constant (≈0.114s per tile) so the jam's existing
  movement-speed tuning carries over instead of inventing a new number.

- `animate_player` (the sprite-frame walk-cycle system) is **unchanged** — it
  already only reacts to `AnimationState.moving`/`facing`, which is now
  reinterpreted as "a `GridMover` tween is in flight" instead of "a key is
  held," and that swap is invisible to it.

## Bug fix: sprite rendered inside the wall when facing left

After the above was in place, a visual bug showed up: standing flush against
a wall while facing left drew the player sprite half-overlapping the wall
tile, even though `GridPos`/collision logic was correct (the player could
not actually step further left).

**Root cause:** facing-left was implemented as `transform.scale.x = -1.0`.
The player sprite uses `Anchor::BOTTOM_LEFT` (to match how tiles are
positioned — `GridPos::to_world()` returns a tile's bottom-left corner), so
normally the sprite's quad extends up-and-right from that anchor point to
fill exactly one tile. Negating `Transform.scale.x` mirrors the *geometry*
around the anchor point, so the quad instead extended up-and-**left** from
that same corner — into whatever tile was to the left, which is the wall
when the player is stopped against it.

**Fix:** flip `Sprite::flip_x` instead of `Transform.scale.x`. `flip_x`
mirrors only the texture's UV sampling (confirmed in
`bevy_sprite::Sprite::compute_pixel_space_point`, which flips
`point_relative_to_sprite_center.x` post-anchor rather than touching the
quad's geometry), so the footprint stays pinned to the current tile and only
the artwork inside it mirrors.

```rust
// before (bug): mirrors geometry around the anchor point
transform.scale.x = if facing == Facing::Left { -1.0 } else { 1.0 };

// after (fix): mirrors only the texture, geometry/anchor unaffected
sprite.flip_x = facing == Facing::Left;
```

## Net behavior change

- Movement: continuous 8-directional analog sliding → discrete 4-directional
  grid steps with a fixed-priority key resolution and a short animated glide.
- Collision: none → hard stop at `Wall` tiles and at the grid's edge.
- Held-key feel: walking multiple tiles in a row glides smoothly, tile to
  tile, with no re-press needed.
- Visual: facing-left no longer causes the sprite to visually spill into the
  adjacent tile.

## Verification performed

- `cargo build`: clean (only pre-existing, unrelated dead-code warnings in
  `level.rs` and one pre-existing `unused_assignments` warning in
  `animate_player`, none introduced by this change).
- `rustfmt --check` on `player.rs`: clean.
- Ran the game briefly (`cargo run`) to confirm no immediate panic.

## Not verified here (needs a manual playtest)

- Visual feel of the tile-to-tile glide and animation pacing
  (`STEP_DURATION_SECS` ≈ `SECONDS_PER_FRAME`, so each step may only show
  1-2 animation frames — flagged as a tuning risk in the original plan).
- Walking into all 4 walls, and that the flip_x fix looks correct in game
  (only reasoned about from the Bevy source, not screenshotted again).
- `cargo clippy` (not available in this shell outside the Nix dev shell).

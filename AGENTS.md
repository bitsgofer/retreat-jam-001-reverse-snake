# AGENTS.md

Guidance for AI agents (and humans) working in this repo.

## What this is

`reverse-snake` is a small game built for BitsGofer Studio's Retreat Game
Jam #001. Design doc / full brief: see `README.md`'s linked Google Doc.

BitsGofer's jams are informal, low-commitment creative retreats — people
hang out and build something together for the fun and learning of it, not
for shipping to real players.

Concept: classic Snake, but reversed —
- The snake is controlled by the computer, not the player.
- The player controls one of many food pieces (concept art frames this as
  a mouse fleeing the snake).
- Otherwise the usual Snake rules apply: bounded 2D grid, snake grows by
  eating food, snake dies if its head hits a wall or its own body.

This is a low-stakes game jam project — favor small, working increments
over polish or heavy abstraction.

### Participants & roles

- @exklamationmark — production, game design, programming, art, sound
- @minhnhdo — programming

Art uses Aseprite; treat art/sound integration as @exklamationmark's lane
unless told otherwise.

### AI disclosure

AI usage is allowed (this isn't being published to real gamers), but
leaning on it too heavily trades away both the joy of building this and
the learning that comes from doing it by hand. Keep changes proportionate
— don't use AI to generate large swaths of code or design decisions that
a human on the team hasn't actually thought through.

When asked to do big, complext task, push back. Break down how those big
tasks should be divided up, and suggest that the human give them a try.

## Stack

- Rust, [Bevy](https://bevyengine.org/) `0.19.1` (game engine/ECS)
- Aseprite (pixel art / sprite sheets)
- Nix flake for the dev environment
- `just` as the task runner

## Layout

- `flake.nix` / `flake.lock` — Nix dev shell (rustc, cargo, just, Bevy's
  native deps: wayland, alsa, vulkan-loader, mesa, etc.)
- `Justfile` — task runner; run `just` with no args to list recipes
- `Cargo.toml` — workspace root (`resolver = "2"`), workspace-level deps
  pinned in `[workspace.dependencies]`
- `game/` — the `reverse-snake` crate (the only workspace member)
  - `game/src/main.rs` — app entry point, window/camera setup. Only wires
    up `PlayerPlugin` via `.add_plugins(...)` so far.
  - `game/src/player.rs` — `PlayerPlugin`: player movement + sprite
    animation (loads `player.png`, a 16-frame x 3-row spritesheet; rows
    are down/up/left-right, with left vs. right done via horizontal flip
    rather than a separate row)
  - `game/src/map.rs` — `MapPlugin`: tile-grid groundwork (`TileGrid`,
    `Tile`, `SpriteSheetKind`) for grass/dirt/road tiles, loading
    `exteriors/1_Terrains_and_Fences_16x16.png` and
    `exteriors/2_City_Terrains_16x16.png`. **Not yet wired into
    `main.rs`** (no `mod map;`/`.add_plugins(MapPlugin)`) — work in
    progress, and those `exteriors/*` textures aren't in `assets/` yet
    either.
- `assets/` — game-specific art, Git-LFS tracked (see `.gitattributes`).
  Currently just `player.aseprite` (source) and `player.png` (exported
  spritesheet used by `player.rs`).
- `references/` — third-party reference art, not used directly by game
  code (see `CREDITS.md` for attribution); also Git-LFS tracked.
- `CREDITS.md` — attribution for third-party assets.
- `.gitignore` / `.gitattributes` — ignores `/target`, `/.direnv`; routes
  `assets/**` and `references/**` through Git LFS.
- `target/`, `Cargo.lock` — build artifacts / lockfile.

Assets referenced by code are expected under `$BEVY_ASSET_ROOT`, which the
Nix shell hook sets to the repo root. `map.rs`'s `exteriors/*` textures
aren't in `assets/` yet, so if/once `MapPlugin` gets wired up, running the
game will warn about those specific missing textures until that art is
added.

## Environment setup

This project expects the Nix dev shell for its native dependencies
(wayland, alsa, vulkan, etc.):

```
nix develop      # or: just dev
```

The shell hook sets `BEVY_ASSET_ROOT` to the repo root, and prints
the `just` recipe list.

## Common commands

Run these via `just` (check `just --list` if this drifts from the recipes
below):

- `just build` — `cargo build`
- `just run` — `cargo run -p reverse-snake`
- `just fmt` / `just fmt-all` — format Rust (`rustfmt`), plus Justfile and
  `*.nix` files when using `fmt-all`
- `just lint` — `cargo fmt --check` + `cargo clippy --workspace -- -D
  warnings`
- `just package` — build itch.io-ready zips for Linux (`dist/reverse-snake-linux.zip`)
  and the browser (`dist/reverse-snake-web.zip`, WASM); `just package-linux` /
  `just package-web` build just one. See `[profile.release]` in the root
  `Cargo.toml` and the `release` group in the Justfile for how these work
  (de-nixifying the Linux binary with `patchelf`, `wasm-bindgen` +
  `wasm-opt` for the web build).

There is no test suite or CI configured yet. Always run `just lint`
before considering Rust changes done.

## Conventions

- Justfile recipes are grouped with `[group("...")]` tags (`build`, `run`,
  `test`, `fmt`, `lint`, `utils`) — keep new recipes in the matching
  section, and keep `[private]` on helper recipes not meant to be run
  directly.
- `flake.nix` and the Justfile use tabs for indentation in places (mixed
  with nixfmt's spaces) — don't fight this by reformatting wholesale;
  `just fmt-nix` / `just fmt-just` own their own formatting.
- Bevy plugins live one-per-file under `game/src/` (see `player.rs`,
  `map.rs`) and are wired up in `main.rs` via `.add_plugins(...)` — check
  `main.rs` for which plugins are actually active before assuming a
  module is live.
- This is a jam project: AI usage is fine, but avoid over-scoping — prefer
  the smallest change that gets a mechanic working over speculative
  abstractions (see AI disclosure above).

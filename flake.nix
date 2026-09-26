{
  description = "BitsGofer Studio Game Jam 001 - Reverse Snake";
  inputs = {
    nixpkgs = {
      url = "github:NixOS/nixpkgs/nixos-unstable";
    };
  };
  outputs =
    { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});
    in
    {
      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          packages = [
            pkgs.just
            pkgs.git-lfs
            pkgs.shfmt
            pkgs.shellcheck
            pkgs.rustc
            pkgs.cargo
            pkgs.nixfmt
            pkgs.pkg-config
            pkgs.wayland
            pkgs.alsa-lib
            pkgs.alsa-plugins
            pkgs.pipewire
            pkgs.udev
            pkgs.libxkbcommon
            pkgs.vulkan-loader
            pkgs.mesa
            pkgs.libdrm
          ];

          shellHook = ''
            export JUST_EXPLAIN=true

            repotRoot="$(pwd)"

            # Assets path are relative to BEVY_ASSET_ROOT
            export BEVY_ASSET_ROOT="$repotRoot"

            # Bevy/winit dlopen() these at runtime (e.g. libxkbcommon for
            # Wayland keyboard input) instead of linking them at build time,
            # so having them in `packages` above isn't enough on non-NixOS
            # hosts without a system-wide copy. Point the dynamic linker at
            # the Nix store paths explicitly.
            export LD_LIBRARY_PATH="${
              pkgs.lib.makeLibraryPath [
                pkgs.wayland
                pkgs.alsa-lib
                pkgs.alsa-plugins
                pkgs.pipewire
                pkgs.udev
                pkgs.libxkbcommon
                pkgs.vulkan-loader
                pkgs.mesa
                pkgs.libdrm
              ]
            }:$LD_LIBRARY_PATH"
          ''
          + ''
            just
            echo ""
            echo "Run: just --usage [recipe] => individual recipe's usage."
            echo "You are now using the development environment!"
            echo ""
          '';
        };
      });
    };
}

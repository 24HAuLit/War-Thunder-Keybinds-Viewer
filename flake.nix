{
  description = "War Thunder Keybind Viewer development environment";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { nixpkgs, ... }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
    in {
      devShells = forAllSystems (system:
        let
          pkgs = import nixpkgs { inherit system; };
          runtimeLibraries = with pkgs; [
            libGL libxkbcommon wayland
            libx11 libxcursor libxi libxrandr
          ];
        in {
          default = pkgs.mkShell {
            packages = with pkgs; [
              cargo rustc rustfmt clippy rust-analyzer pkg-config
              # Headless smoke tests of the native window.
              xorg-server xdotool
            ];
            buildInputs = runtimeLibraries;
            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath runtimeLibraries;
            RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
          };
        });
    };
}

{pkgs ? import <nixpkgs> {}}:
# Build the niri-switch Rust project with GTK4 support.
# Usage (non-flake):
#   nix-build
# Or to run the binary after build:
#   result/bin/niri-switch --help
let
  inherit (pkgs) lib;
in
  pkgs.rustPlatform.buildRustPackage {
    pname = "niri-switch";
    version = "0.1.0";

    src = ./.;

    # Use the project's Cargo.lock.
    cargoLock = {
      lockFile = ./Cargo.lock;
    };

    # Tools required at build time
    nativeBuildInputs = [
      pkgs.pkg-config
      pkgs.wrapGAppsHook4
      pkgs.glib # for glib-compile-resources used by build.rs (glib-build-tools)
    ];

    # System libraries the Rust GTK bindings link against
    buildInputs = [
      pkgs.glib
      pkgs.gtk4
      pkgs.gtk4-layer-shell
      pkgs.gsettings-desktop-schemas
    ];

    # The main CLI the user is likely to run
    meta = with lib; {
      description = "A fast task switcher for the niri compositor";
      license = licenses.gpl3Plus;
      homepage = "https://github.com/Kiki-Bouba-Team/niri-switch";
      maintainers = [maintainers.cezaryswitala];
      mainProgram = "niri-switch";
      platforms = platforms.linux;
    };
  }

{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs = inputs @ {flake-parts, ...}: let
    inherit (cargoToml.package) name version;
    cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
  in
    flake-parts.lib.mkFlake {inherit inputs;} ({...}: {
      systems = ["x86_64-linux" "aarch64-linux"];
      perSystem = {pkgs, ...}: {
        devShells = {
          # nix develop
          default = pkgs.mkShell {
            nativeBuildInputs = with pkgs; [
              rustc
              cargo
              rust-analyzer
              pkg-config
              gtk4
              gtk4-layer-shell
              gsettings-desktop-schemas
              glib
            ];
          };
        };

        packages = {
          default = pkgs.callPackage ./nix {inherit version name;};
        };
      };
    });
}

{
  rustPlatform,
  lib,
  name,
  version,
  pkg-config,
  gtk4,
  gtk4-layer-shell,
  gsettings-desktop-schemas,
  wrapGAppsHook4,
  glib,
}:
rustPlatform.buildRustPackage {
  inherit version;
  pname = name;
  cargoLock.lockFile = ../Cargo.lock;
  src = lib.cleanSource ../.;

  nativeBuildInputs = [
    pkg-config
    wrapGAppsHook4
    glib
  ];

  buildInputs = [
    glib
    gtk4
    gtk4-layer-shell
    gsettings-desktop-schemas
  ];

  meta = with lib; {
    description = "A fast task switcher for the niri compositor";
    license = licenses.gpl3Plus;
    homepage = "https://github.com/Kiki-Bouba-Team/niri-switch";
    maintainers = [maintainers.cezaryswitala];
    mainProgram = "niri-switch-daemon";
    platforms = platforms.linux;
  };
}

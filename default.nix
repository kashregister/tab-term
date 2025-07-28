{pkgs ? import <nixpkgs> {}}:
pkgs.rustPlatform.buildRustPackage rec {
  pname = "ttime";
  version = "0.1.0";
  cargoLock.lockFile = ./Cargo.lock;
  src = pkgs.lib.cleanSource ./.;

  buildInputs = with pkgs; [
    dbus
    alsa-lib
    pkg-config
    libressl
  ];

  nativeBuildInputs = with pkgs; [
    dbus
    alsa-lib
    pkg-config
  ];
}

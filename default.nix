{
  rustPlatform,
  lib,
}:
rustPlatform.buildRustPackage
rec {
  pname = (lib.importTOML "${src}/Cargo.toml").package.name;
  version = (lib.importTOML "${src}/Cargo.toml").package.version;
  src = ./.;

  cargoLock.lockFile = "${src}/Cargo.lock";
}

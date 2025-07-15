{
  lib,
  pkgs,
  rev ? "dirty",
}:
let
  cargoToml = builtins.fromTOML (builtins.readFile ../Cargo.toml);
in
pkgs.rustPlatform.buildRustPackage {
  inherit (cargoToml.package) name;
  version = "${cargoToml.package.version}-${rev}";

  nativeBuildInputs = with pkgs; [ pkg-config ];
  buildInputs = with pkgs; [
    rust-bin.stable.latest.default
    yt-dlp
    openssl
    libopus
  ];
  src = lib.cleanSource ../.;
  cargoLock.lockFile = ../Cargo.lock;

  meta = with lib; {
    description = "rauly.rs discord bot";
    homepage = "https://github.com/frahz/rauly.rs";
    licenses = licenses.mit;
    mainProgram = "raulyrs";
  };
}

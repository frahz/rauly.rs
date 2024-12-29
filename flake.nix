{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";

    pre-commit-hooks.url = "github:cachix/pre-commit-hooks.nix";
    pre-commit-hooks.inputs.nixpkgs.follows = "nixpkgs";
  };
  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
    pre-commit-hooks,
  }:
    flake-utils.lib.eachDefaultSystem
    (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [(import rust-overlay)];
        };
        cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);

        nativeBuildInputs = with pkgs; [pkg-config];
        buildInputs = with pkgs; [
          rust-bin.stable.latest.default
          yt-dlp
          openssl
          libopus
        ];
      in
        with pkgs; {
          checks = {
            pre-commit-check = pre-commit-hooks.lib.${system}.run {
              src = ./.;
              hooks = {
                rustfmt.enable = true;
              };
            };
          };
          packages = {
            default = rustPlatform.buildRustPackage {
              inherit nativeBuildInputs buildInputs;
              inherit (cargoToml.package) name version;
              src = lib.cleanSource ./.;
              cargoLock.lockFile = ./Cargo.lock;

              meta = with lib; {
                description = "rauly.rs discord bot";
                homepage = "https://github.com/frahz/rauly.rs";
                licenses = licenses.mit;
                mainProgram = "raulyrs";
              };
            };
            raulyrs = self.packages.${system}.default;
          };
          devShells.default = mkShell {
            inherit (self.checks.${system}.pre-commit-check) shellHook;
            inherit nativeBuildInputs buildInputs;
          };
        }
    )
    // {
      overlays.default = final: _: {
        raulyrs = final.callPackage self.packages.${final.system}.default {};
      };
      nixosModules.default = import ./nix/module.nix self;
    };
}

{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
    pre-commit-hooks = {
      url = "github:cachix/pre-commit-hooks.nix";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
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
          packages.default = rustPlatform.buildRustPackage {
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
      nixosModules.default = {
        config,
        lib,
        pkgs,
        ...
      }:
        with lib; let
          cfg = config.services.raulyrs;
        in {
          options.services.raulyrs = {
            enable = mkEnableOption "rauly.rs discord bot";
            package = mkOption {
              type = types.package;
              default = pkgs.raulyrs;
              description = ''
                Package for rauly.rs discord bot
              '';
            };
            environmentFile = mkOption {
              type = types.path;
              description = ''
                Path containing the Bot's API keys
              '';
            };
          };
          config = mkIf cfg.enable {
            systemd.services.raulyrs = {
              description = "rauly.rs discord bot";
              wantedBy = ["multi-user.target"];
              path = [ pkgs.yt-dlp ];
              serviceConfig = {
                Type = "simple";
                User = "raulyrs";
                ExecStart = lib.getExe cfg.package;
                EnvironmentFile = cfg.environmentFile;
              };
            };

            users = {
              users.raulyrs = {
                description = "rauly.rs service user";
                isSystemUser = true;
                group = "raulyrs";
              };
              groups.raulyrs = {};
            };
          };
        };
    };
}

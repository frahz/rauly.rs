{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";

    pre-commit-hooks.url = "github:cachix/pre-commit-hooks.nix";
    pre-commit-hooks.inputs.nixpkgs.follows = "nixpkgs";
  };
  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    pre-commit-hooks,
  }: let
    systems = [
      "x86_64-linux"
      "aarch64-linux"
      # "aarch64-darwin"
    ];
    forEachSystem = nixpkgs.lib.genAttrs systems;
    pkgsForEach = forEachSystem (system:
      import nixpkgs {
        inherit system;
        overlays = [rust-overlay.overlays.default];
      });
  in {
    checks = forEachSystem (system: {
      pre-commit-check = pre-commit-hooks.lib.${system}.run {
        src = ./.;
        hooks = {
          rustfmt.enable = true;
        };
      };
    });

    devShells = forEachSystem (system: let
      pkgs = pkgsForEach.${system};
    in {
      default = pkgs.callPackage ./nix/shell.nix {
        inherit pkgs;
        inherit (self.checks.${system}.pre-commit-check) shellHook;
      };
    });

    packages = forEachSystem (system: let
      pkgs = pkgsForEach.${system};
    in {
      raulyrs = pkgs.callPackage ./nix/package.nix {};
      default = self.packages.${system}.raulyrs;
    });

    nixosModules.default = import ./nix/module.nix self;
  };
}

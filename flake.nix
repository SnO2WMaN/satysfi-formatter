{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nix-filter.url = "github:numtide/nix-filter";

    # dev
    devshell.url = "github:numtide/devshell";
    flake-utils.url = "github:numtide/flake-utils";
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };
  outputs = {
    self,
    nixpkgs,
    nix-filter,
    flake-utils,
    naersk,
    devshell,
    ...
  } @ inputs:
    {
      overlays.default = final: prev: let
        naersk' = final.callPackage naersk {};
      in {
        satysfi-formatter = naersk'.buildPackage {
          pname = "satysfi-formatter";
          root = with nix-filter.lib;
            filter {
              root = ./.;
              include = [
                "Cargo.toml"
                "Cargo.lock"
                (inDirectory "src")
              ];
            };
        };
        satysfi-formatter-write-each = final.callPackage ./nix/fmt-write-each.nix {};
      };
    }
    // flake-utils.lib.eachDefaultSystem (
      system: let
        inherit (pkgs) lib;
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            devshell.overlay
            self.overlays.default
          ];
        };
        naersk' = pkgs.callPackage naersk {};
      in {
        packages.satysfi-formatter = pkgs.satysfi-formatter;
        packages.satysfi-formatter-write-each = pkgs.satysfi-formatter-write-each;
        packages.default = self.packages.${system}.satysfi-formatter;

        # `nix run`
        apps.satysfi-formatter = flake-utils.lib.mkApp {
          drv = self.packages.${system}.satysfi-formatter;
          name = "satysfi-fmt";
        };
        apps.satysfi-formatter-write-each = flake-utils.lib.mkApp {
          drv = self.packages.${system}.satysfi-formatter-each;
          name = "satysfi-fmt-write-each";
        };
        apps.default = self.apps.${system}.satysfi-formatter;

        checks = {
          inherit
            (self.packages.${system})
            satysfi-formatter
            satysfi-formatter-write-each
            ;
        };

        # `nix develop`
        devShell = pkgs.devshell.mkShell {
          commands = with pkgs; [
            {
              package = "treefmt";
              category = "formatter";
            }
          ];
          packages = with pkgs;
            [
              gcc
              cargo
              rustc

              # develop
              cargo-make
              python3
              alejandra
              taplo-cli
              rustfmt
            ]
            ++ (with self.packages.${system}; [
              satysfi-formatter
              satysfi-formatter-write-each
            ]);
        };
      }
    );
}

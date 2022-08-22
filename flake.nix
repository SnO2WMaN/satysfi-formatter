{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };

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
    flake-utils,
    naersk,
    devshell,
    ...
  } @ inputs:
    flake-utils.lib.eachDefaultSystem (
      system: let
        inherit (pkgs) lib;
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            devshell.overlay
          ];
        };
        naersk' = pkgs.callPackage naersk {};
      in {
        packages.satysfi-formatter = naersk'.buildPackage {
          pname = "satysfi-formatter";
          root = builtins.path {
            path = ./.;
            filter = name: type:
              (lib.hasPrefix (toString ./src) name)
              || (name == toString ./Cargo.toml)
              || (name == toString ./Cargo.lock);
          };
        };
        packages.satysfi-formatter-write-each = pkgs.callPackage ./nix/fmt-write-each.nix {
          satysfi-formatter = self.packages.${system}.satysfi-formatter;
        };
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

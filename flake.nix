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
    self, nixpkgs, flake-utils, naersk, devshell, ... } @ inputs:
    flake-utils.lib.eachDefaultSystem (system:
      let
        inherit (pkgs) lib;
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ devshell.overlay ];
        };
        naersk' = pkgs.callPackage naersk {};
      in
      rec {
        # `nix build`
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
        packages.default = packages.satysfi-formatter;

        # `nix run`
        apps.satysfi-formatter = flake-utils.lib.mkApp {
          drv = packages.satysfi-formatter;
          name = "satysfi-fmt";
        };
        apps.default = apps.satysfi-formatter;

        # `nix develop`
        devShell = pkgs.devshell.mkShell {
          commands = with pkgs; [
            {
              package = "treefmt";
              category = "formatter";
            }
          ];
          packages = with pkgs; [
            gcc
            cargo
            rustc

            # develop
            cargo-make
            python3
            alejandra
            taplo-cli
            rustfmt
          ];
        };
      }
    );
}

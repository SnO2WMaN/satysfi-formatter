{
  inputs = {
    nixpkgs = {
      url = "github:nixos/nixpkgs/nixos-unstable";
    };
    flake-utils = {
      url = "github:numtide/flake-utils";
    };
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    devshell = {
      url = "github:numtide/devshell";
    };
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };
  outputs = { self, nixpkgs, flake-utils, naersk, devshell, ... } @ inputs:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ devshell.overlay ];
        };
        naersk-lib = naersk.lib."${system}";
      in
      rec {
        # `nix build`
        packages.satysfi-formatter = naersk-lib.buildPackage {
          pname = "satysfi-formatter";
          root = ./.;
        };
        defaultPackage = packages.satysfi-formatter;

        # `nix run`
        apps.satysfi-formatter = flake-utils.lib.mkApp {
          drv = packages.satysfi-formatter;
        };
        defaultApp = apps.satysfi-formatter;

        # `nix develop`
        devShell = pkgs.devshell.mkShell {
          imports = [
            (pkgs.devshell.importTOML ./devshell.toml)
          ];
          packages = [
            packages.satysfi-formatter
          ];
        };
      });
}

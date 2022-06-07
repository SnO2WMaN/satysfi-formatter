{
  inputs = {
    nixpkgs = {
      url = "github:nixos/nixpkgs/nixos-unstable";
    };
    nci = {
      url = "github:yusdacra/nix-cargo-integration";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };
  outputs = { self, nixpkgs, nci, ... } @ inputs:
    inputs.nci.lib.makeOutputs {
      root = ./.;
      overrides = {
        shell = common: prev: {
          packages =
            prev.packages
            ++ (with common.pkgs; [
              cargo-edit
              cargo-make
              cargo-watch
              nixpkgs-fmt
              rust-analyzer
              taplo-cli
              treefmt
            ]);
          commands = prev.commands;
          env = prev.env;
        };
      };
    };
}

{
  description = "embassy flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix.url = "github:nix-community/fenix/monthly";
  };

  outputs = { self, nixpkgs, flake-utils, fenix }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
        fpkgs = fenix.packages.${system};
        profile = fpkgs.complete;
        std-lib = fpkgs.targets.thumbv7em-none-eabihf.latest;
        rust-analyzer-nightly = fpkgs.rust-analyzer;
        rust-toolchain = fpkgs.combine [
          profile.rustc
          profile.rust-src
          profile.cargo
          profile.rustfmt
          profile.clippy
          profile.llvm-tools
          std-lib.rust-std
        ];
      in
      {
        devShells.default =
        pkgs.mkShell {
          buildInputs = with pkgs; [
            rust-toolchain
            rust-analyzer-nightly

            # extra cargo tools
            cargo-edit
            cargo-expand
            cargo-show-asm
            cargo-binutils
          ];

          # set the rust src for rust_analyzer
          RUST_SRC_PATH = "${rust-toolchain}/lib/rustlib/src/rust/library";
        };
      }
    );
}


{
  description = "Hardened Wasmtime profile admission, normalized outcomes, and strategy-parity rail";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/20535e48e12c86043b577b8518234ff5dbb26957";
    flake-utils.url = "github:numtide/flake-utils/11707dc2f618dd54ca8739b309ec4fc024de578b";
    rust-overlay = {
      url = "github:oxalica/rust-overlay/3c38e1e1ba9c8d7030f7b5a801398ea7d8a6fdc0";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane/b556d7bbae5ff86e378451511873dfd07e4504cd";
    differential-execution = {
      url = "git+https://seed.radicle.garden/z2pbwQ4Zd52M4oC1W9bLT3nQem9ay.git?rev=cad5a7cb6c6502b0ec5dc76c51e03352260d8efd";
      flake = false;
    };
    cairn = {
      url = "github:OnixResearch/cairn/b6759012db4dab0158936e995b7bad39ac10465b";
      flake = false;
    };
    octet = {
      url = "github:OnixResearch/octet?rev=374bd16b26cee2af34211a29bfa531c016811f51";
      inputs.crane.follows = "crane";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
      crane,
      differential-execution,
      cairn,
      octet,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
        source = pkgs.lib.cleanSource self;

        cargoVendorDir = craneLib.vendorCargoDeps {
          cargoLock = ./Cargo.lock;
          # Source evidence contains historical store-path text, not runtime
          # references. Use the hash-locked flake source rather than fetchgit's
          # fixed-output derivation, which rejects that text as a reference.
          overrideVendorGitCheckout =
            packages: checkout:
            if builtins.all (package: package.name == "differential-execution-core") packages then
              checkout.overrideAttrs (_old: { src = differential-execution; })
            else
              checkout;
        };

        commonArgs = {
          pname = "hardened-wasmtime-runtime";
          version = "0.1.0";
          src = source;
          inherit cargoVendorDir;
          strictDeps = true;
        };
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        crate = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;
          }
        );

        octetConfig =
          (builtins.fromJSON (builtins.readFile ./config/octet-deny-all.json)).config;
        octetFlags = builtins.concatStringsSep " " (map (lint: "-D ${lint}") octetConfig.lints);
        octetRunner = pkgs.writeShellApplication {
          name = "octet-deny-all";
          runtimeInputs = [ octet.packages.${system}.cargo-octet ];
          text = ''
            export DYLINT_RUSTFLAGS=${pkgs.lib.escapeShellArg octetFlags}
            exec cargo-octet check "$@"
          '';
        };
        octetCheck =
          (octet.lib.mkConsumerCheck {
            inherit system;
            src = self;
            cargoLock = ./Cargo.lock;
          }).overrideAttrs
            (_old: {
              DYLINT_RUSTFLAGS = octetFlags;
            });

        tests = craneLib.cargoTest (
          commonArgs
          // {
            inherit cargoArtifacts;
            cargoTestCommand = "cargo test --workspace --all-targets";
          }
        );
      in
      {
        packages = {
          default = crate;
          inherit crate;
        };

        apps = {
          octet-deny-all = {
            type = "app";
            program = "${octetRunner}/bin/octet-deny-all";
          };
        };

        checks = {
          inherit tests;
          octet = octetCheck;
          build = crate;
        };

        devShells.default = pkgs.mkShell {
          packages = [
            rustToolchain
            octet.packages.${system}.cargo-octet
            pkgs.wasmtime
            pkgs.b3sum
            pkgs.jq
          ];
          shellHook = ''
            echo "hardened-wasmtime-runtime development environment"
            echo "Rust: $(rustc --version)"
            echo ""
            echo "Gates:"
            echo "  cargo test"
            echo "  nix run .#octet-deny-all -- --workspace -- --all-targets"
            echo "  nix flake check -L"
          '';
        };
      }
    );
}

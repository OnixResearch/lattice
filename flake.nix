{
  description = "Lattice — graph-native workflow automation built with Rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    octet.url = "github:OnixResearch/octet";
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      rust-overlay,
      flake-utils,
      octet,
      ...
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

        # Common source filtering
        sourceRoot = toString ./.;
        isSourceFixturePath =
          path:
          pkgs.lib.hasPrefix "${sourceRoot}/examples/" (toString path)
          || pkgs.lib.hasPrefix "${sourceRoot}/tests/" (toString path)
          || pkgs.lib.hasPrefix "${sourceRoot}/cairn/specs/" (toString path)
          || pkgs.lib.hasPrefix "${sourceRoot}/cairn-policy/" (toString path);
        isAnimusCompileTimeAssetPath =
          path:
          pkgs.lib.hasPrefix "${sourceRoot}/vendor-cargo-deps/animus/config/" (toString path)
          || pkgs.lib.hasPrefix "${sourceRoot}/vendor-cargo-deps/animus/policy/" (toString path);
        isRatsCompileTimeAssetPath =
          path:
          toString path == "${sourceRoot}/vendor-cargo-deps/rats/source/SOURCE"
          || pkgs.lib.hasPrefix "${sourceRoot}/vendor-cargo-deps/rats/source/docs/generated/snapshots/" (
            toString path
          );
        isRepositoryValidationAssetPath =
          path: toString path == "${sourceRoot}/.gitignore";
        isIdentityAuditAssetPath =
          path:
          toString path == "${sourceRoot}/README.md"
          || toString path == "${sourceRoot}/flake.nix"
          || pkgs.lib.hasPrefix "${sourceRoot}/scripts/" (toString path)
          || pkgs.lib.hasPrefix "${sourceRoot}/cairn/archive/2026-07-20-rename-n8nclone-to-lattice/" (
            toString path
          );
        src = pkgs.lib.cleanSourceWith {
          src = ./.;
          filter =
            path: type:
            craneLib.filterCargoSources path type
            || isSourceFixturePath path
            || isAnimusCompileTimeAssetPath path
            || isRatsCompileTimeAssetPath path
            || isRepositoryValidationAssetPath path
            || isIdentityAuditAssetPath path;
        };

        # Common build inputs
        nativeBuildInputs = with pkgs; [
          pkg-config
          clang
          mold
          git
        ];

        buildInputs =
          with pkgs;
          [
            openssl
          ]
          ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.darwin.apple_sdk.frameworks.Security
            pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
          ];

        # Build just the cargo dependencies for caching
        cargoArtifacts = craneLib.buildDepsOnly {
          inherit src nativeBuildInputs buildInputs;
        };

        octetPreCommitCheck = pkgs.writeShellApplication {
          name = "octet-precommit-check";
          runtimeInputs =
            (with pkgs; [
              coreutils
              gnugrep
              jq
            ])
            ++ [ octet.packages.${system}.cargo-octet ];
          text = ''
            success_status=0
            lint_failure_status=1
            octet_status="$success_status"
            artifact_dir="''${OCTET_ARTIFACT_DIR:-target/octet-precommit}"
            cargo_target_dir="''${OCTET_CARGO_TARGET_DIR:-target/octet-cargo-target}"
            baseline_file="''${OCTET_BASELINE:-octet-baseline.json}"
            status_file="$artifact_dir/status.json"
            summary_file="$artifact_dir/summary.txt"

            rm -rf "$artifact_dir"
            mkdir -p "$artifact_dir"

            baseline_args=()
            if [ -f "$baseline_file" ]; then
              baseline_args=(--baseline "$baseline_file")
            fi

            set +e
            CARGO_TARGET_DIR="$cargo_target_dir" cargo-octet check --output-format json --artifact-dir "$artifact_dir" "''${baseline_args[@]}" > "$artifact_dir/stdout.json"
            octet_status="$?"
            set -e

            if [ ! -f "$status_file" ]; then
              exit "$octet_status"
            fi

            if jq --exit-status '
              .status == "clean"
              or (
                .baseline != null
                and (.baseline.new_total | tostring) == "0"
                and (.baseline.current_total | tonumber) > 0
                and (.baseline.accepted_current | tonumber) > 0
              )
            ' "$status_file" > /dev/null; then
              exit "$success_status"
            fi

            if [ -f "$summary_file" ]; then
              cat "$summary_file" >&2
            else
              cat "$status_file" >&2
            fi

            if jq --exit-status '.baseline != null' "$status_file" > /dev/null; then
              jq --raw-output '
                "Status: "
                + (.status | tostring)
                + ", cargo="
                + (.cargo_process_exit.classification | tostring)
                + ", baseline: "
                + (.baseline.accepted_total | tostring)
                + " accepted, "
                + (.baseline.accepted_current | tostring)
                + " current accepted, "
                + (.baseline.new_total | tostring)
                + " new, "
                + (.baseline.stale_total | tostring)
                + " stale"
              ' "$status_file" >&2
            fi
            exit "$lint_failure_status"
          '';
        };

        octetVendorDir = craneLib.vendorCargoDeps { cargoLock = ./Cargo.lock; };

        octetCheck =
          pkgs.runCommand "octet-consumer-check"
            {
              nativeBuildInputs = nativeBuildInputs ++ [ octetPreCommitCheck ];
              inherit buildInputs;
            }
            ''
              export HOME="$PWD/home"
              mkdir -p "$HOME"
              cp -r ${self} source
              chmod -R u+w source
              cd source

              export CARGO_HOME="$PWD/.cargo-home"
              mkdir -p "$CARGO_HOME"
              cp "${octetVendorDir}/config.toml" "$CARGO_HOME/config.toml"

              OCTET_ARTIFACT_DIR="$PWD/target/octet-check" octet-precommit-check
              mkdir -p "$out"
            '';

        traceabilityInventoryPolicyCheck =
          pkgs.runCommand "lattice-traceability-inventory-policy"
            {
              nativeBuildInputs = [ pkgs.nickel ];
            }
            ''
              unexpected_success=1
              nickel export --format json ${src}/cairn-policy/traceability-inventory.ncl > inventory-policy.json
              if nickel export --format json ${src}/cairn-policy/fixtures/invalid-traceability-inventory.ncl > /dev/null 2>&1; then
                echo "invalid traceability inventory policy unexpectedly exported" >&2
                exit "$unexpected_success"
              fi
              touch "$out"
            '';

        # Build the actual package
        lattice = craneLib.buildPackage {
          inherit
            src
            cargoArtifacts
            nativeBuildInputs
            buildInputs
            ;
        };
      in
      {
        packages = {
          default = lattice;
          lattice = lattice;
        };

        checks = {
          inherit lattice;

          # Run tests with nextest
          nextest = craneLib.cargoNextest {
            inherit
              src
              cargoArtifacts
              nativeBuildInputs
              buildInputs
              ;
            partitions = 1;
            partitionType = "count";
          };

          # Clippy lints
          clippy = craneLib.cargoClippy {
            inherit
              src
              cargoArtifacts
              nativeBuildInputs
              buildInputs
              ;
            cargoClippyExtraArgs = "--package lattice --all-targets --no-deps -- -D warnings";
          };

          # Format check
          fmt = craneLib.cargoFmt {
            inherit src;
            cargoExtraArgs = "--package lattice";
          };

          # Octet Tiger Style lints
          octet = octetCheck;

          # Typed traceability inventory policy and negative fixture
          traceability-inventory-policy = traceabilityInventoryPolicyCheck;
        };

        devShells.default = craneLib.devShell {
          inherit buildInputs;

          packages =
            (with pkgs; [
              cargo-nextest
              cargo-watch
              pre-commit
              rust-analyzer
            ])
            ++ [
              octet.packages.${system}.cargo-octet
              octetPreCommitCheck
            ];

          # Ensure the nightly toolchain is available
          inputsFrom = [ lattice ];
        };
      }
    );
}

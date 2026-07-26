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
    bounded-exec = {
      url = "git+https://git.onix.computer/z2CpqLFpdP36fZXYUK5ZNWxMibpCo.git?rev=29dac88ecded94457572db3fdfaaaab95fa91525";
      flake = false;
    };
    execution-graph = {
      url = "git+https://git.onix.computer/z2oYsb9jGTyp68BKYhzpivY1eK58a.git?rev=03736f1ec46c377ff86b451260ad68aa70ff3b0b";
      flake = false;
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      rust-overlay,
      flake-utils,
      octet,
      bounded-exec,
      execution-graph,
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
        isRepositoryValidationAssetPath = path: toString path == "${sourceRoot}/.gitignore";
        isIdentityAuditAssetPath =
          path:
          toString path == "${sourceRoot}/README.md"
          || toString path == "${sourceRoot}/flake.nix"
          || toString path == "${sourceRoot}/flake.lock"
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

        boundedExecCargoSource = "git+https://git.onix.computer/z2CpqLFpdP36fZXYUK5ZNWxMibpCo.git?rev=29dac88ecded94457572db3fdfaaaab95fa91525#29dac88ecded94457572db3fdfaaaab95fa91525";
        executionGraphCargoSource = "git+https://git.onix.computer/z2oYsb9jGTyp68BKYhzpivY1eK58a.git?rev=03736f1ec46c377ff86b451260ad68aa70ff3b0b#03736f1ec46c377ff86b451260ad68aa70ff3b0b";
        cargoVendorDir = craneLib.vendorCargoDeps {
          inherit src;
          overrideVendorGitCheckout =
            packages: checkout:
            if builtins.all (package: package.source == boundedExecCargoSource) packages then
              checkout.overrideAttrs (_old: {
                src = bounded-exec;
              })
            else if builtins.all (package: package.source == executionGraphCargoSource) packages then
              checkout.overrideAttrs (_old: {
                src = execution-graph;
              })
            else
              checkout;
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
          inherit
            src
            cargoVendorDir
            nativeBuildInputs
            buildInputs
            ;
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

        octetVendorDir = cargoVendorDir;

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

        boundedExecSourceIdentityCheck =
          pkgs.runCommand "lattice-bounded-exec-source-identity"
            {
              nativeBuildInputs = [
                pkgs.b3sum
                pkgs.coreutils
                pkgs.diffutils
                pkgs.jq
                pkgs.nickel
              ];
            }
            ''
              source_root=${bounded-exec}
              test -f "$source_root/Cargo.toml"
              test -f "$source_root/crates/bounded-exec/Cargo.toml"
              test -f "$source_root/crates/bounded-exec-core/Cargo.toml"
              grep --fixed-strings 'license = "AGPL-3.0-or-later"' "$source_root/Cargo.toml" > /dev/null
              grep --fixed-strings 'name = "bounded-exec"' "$source_root/crates/bounded-exec/Cargo.toml" > /dev/null
              grep --fixed-strings 'name = "bounded-exec-core"' "$source_root/crates/bounded-exec-core/Cargo.toml" > /dev/null

              admission=${self}/tests/fixtures/radicle/bounded-exec-source-admission-v1.json
              expected_hash="$(cat ${self}/tests/fixtures/radicle/bounded-exec-source-admission-v1.blake3)"
              actual_hash="$(b3sum "$admission" | cut -d ' ' -f 1)"
              test "$actual_hash" = "$expected_hash"
              nickel export --format json ${self}/tests/fixtures/radicle/bounded-exec-source-admission-v1.ncl \
                | jq -S . > "$TMPDIR/admission-from-nickel.json"
              cmp "$admission" "$TMPDIR/admission-from-nickel.json"
              jq -e \
                --arg rid 'rad:z2CpqLFpdP36fZXYUK5ZNWxMibpCo' \
                --arg rev '29dac88ecded94457572db3fdfaaaab95fa91525' \
                --arg source_hash '4fbbf8f0749262469f00748e04c775180488dba800303f139172656d25931927' \
                '.status == "accepted" and .repository.rid == $rid and .repository.reviewed_commit == $rev and .repository.source_archive_blake3 == $source_hash and .availability.github_fallback_configured == false' \
                "$admission" > /dev/null

              cutover=${self}/evidence/radicle/bounded-exec-cutover-v1.json
              expected_cutover_hash="$(cat ${self}/evidence/radicle/bounded-exec-cutover-v1.blake3)"
              actual_cutover_hash="$(b3sum "$cutover" | cut -d ' ' -f 1)"
              test "$actual_cutover_hash" = "$expected_cutover_hash"
              nickel export --format json ${self}/evidence/radicle/bounded-exec-cutover-v1.ncl \
                | jq -S . > "$TMPDIR/cutover-from-nickel.json"
              cmp "$cutover" "$TMPDIR/cutover-from-nickel.json"
              jq -e \
                --arg rid 'rad:z2CpqLFpdP36fZXYUK5ZNWxMibpCo' \
                --arg rev '29dac88ecded94457572db3fdfaaaab95fa91525' \
                '.status == "accepted" and .repository.rid == $rid and .repository.reviewed_commit == $rev and .cargo.github_fallback_configured == false and .availability.github_fallback_configured == false' \
                "$cutover" > /dev/null

              forbidden_source='github.com/OnixResearch/'"bounded-exec"
              if grep --fixed-strings "$forbidden_source" \
                ${self}/Cargo.toml ${self}/Cargo.lock ${self}/flake.nix ${self}/flake.lock; then
                echo 'Bounded Exec source retained a GitHub fallback' >&2
                exit 1
              fi
              touch "$out"
            '';

        # r[verify lattice.execution_graph_external_adoption.source.scenario.accepted]
        # r[verify lattice.execution_graph_external_adoption.source.scenario.rejected]
        # r[verify lattice.execution_graph_external_adoption.cutover.scenario.accepted]
        executionGraphSourceIdentityCheck =
          pkgs.runCommand "lattice-execution-graph-source-identity"
            {
              nativeBuildInputs = [
                pkgs.b3sum
                pkgs.coreutils
                pkgs.diffutils
                pkgs.jq
                pkgs.nickel
              ];
            }
            ''
              source_root=${execution-graph}
              test -f "$source_root/Cargo.toml"
              test -f "$source_root/src/lib.rs"
              grep --fixed-strings 'name = "execution-graph-core"' "$source_root/Cargo.toml" > /dev/null
              grep --fixed-strings 'license = "AGPL-3.0-or-later"' "$source_root/Cargo.toml" > /dev/null
              test "$(b3sum "$source_root/src/lib.rs" | cut -d ' ' -f 1)" = '91450b94f6256c71057507b6ae97f63e505dc57377198be38087530bedefccb4'
              test ! -e ${self}/vendor-cargo-deps/seaglass-execution-graph

              admission=${self}/tests/fixtures/radicle/execution-graph-source-admission-v1.json
              expected_hash="$(cat ${self}/tests/fixtures/radicle/execution-graph-source-admission-v1.blake3)"
              actual_hash="$(b3sum "$admission" | cut -d ' ' -f 1)"
              test "$actual_hash" = "$expected_hash"
              nickel export --format json ${self}/tests/fixtures/radicle/execution-graph-source-admission-v1.ncl \
                | jq -S . > "$TMPDIR/admission-from-nickel.json"
              cmp "$admission" "$TMPDIR/admission-from-nickel.json"
              jq -e \
                --arg rid 'rad:z2oYsb9jGTyp68BKYhzpivY1eK58a' \
                --arg rev '03736f1ec46c377ff86b451260ad68aa70ff3b0b' \
                --arg source_hash '4b5aa3756369236fc82fbbf501d35993cfa208f142694cdd30ca370d6241192c' \
                '.status == "accepted" and .repository.rid == $rid and .repository.reviewed_commit == $rev and .repository.source_archive_blake3 == $source_hash and .availability.github_fallback_configured == false' \
                "$admission" > /dev/null

              receipt=${self}/evidence/radicle/execution-graph-cutover-v1.json
              expected_receipt_hash="$(cat ${self}/evidence/radicle/execution-graph-cutover-v1.blake3)"
              actual_receipt_hash="$(b3sum "$receipt" | cut -d ' ' -f 1)"
              test "$actual_receipt_hash" = "$expected_receipt_hash"
              nickel export --format json ${self}/evidence/radicle/execution-graph-cutover-v1.ncl \
                | jq -S . > "$TMPDIR/receipt-from-nickel.json"
              cmp "$receipt" "$TMPDIR/receipt-from-nickel.json"
              jq -e \
                --arg cutover 'a85da0afc2f658ca29fcb8ebe6927896ca85e085' \
                '.status == "accepted" and .consumer.cutover_revision == $cutover and .cargo.vendor_present == false and .cargo.fallback_configured == false and .validation.nix_flake_check == "passed-x86_64-linux" and .validation_baseline.new_ids == 0 and .validation_baseline.stale_ids == 0' \
                "$receipt" > /dev/null

              grep --fixed-strings 'seaglass-execution-graph = { package = "execution-graph-core", git = "https://git.onix.computer/z2oYsb9jGTyp68BKYhzpivY1eK58a.git", rev = "03736f1ec46c377ff86b451260ad68aa70ff3b0b" }' ${self}/Cargo.toml > /dev/null
              grep --fixed-strings '${executionGraphCargoSource}' ${self}/Cargo.lock > /dev/null
              jq -e \
                '.nodes["execution-graph"].locked.rev == "03736f1ec46c377ff86b451260ad68aa70ff3b0b" and .nodes["execution-graph"].locked.url == "https://git.onix.computer/z2oYsb9jGTyp68BKYhzpivY1eK58a.git" and .nodes["execution-graph"].locked.narHash == "sha256-SzdJGsS631Hk3CbPLThz+2YPgvdoB1cXb3BPusJTX8I="' \
                ${self}/flake.lock > /dev/null
              touch "$out"
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
              if nickel export --format json ${src}/cairn-policy/fixtures/invalid-traceability-negative-missing-count.ncl > /dev/null 2>&1; then
                echo "negative traceability missing count unexpectedly exported" >&2
                exit "$unexpected_success"
              fi
              touch "$out"
            '';

        # Build the actual package
        lattice = craneLib.buildPackage {
          inherit
            src
            cargoArtifacts
            cargoVendorDir
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
              cargoVendorDir
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
              cargoVendorDir
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

          # Exact published process-runner source identity
          bounded-exec-source = boundedExecSourceIdentityCheck;

          # Exact published graph-mechanism source identity
          execution-graph-source = executionGraphSourceIdentityCheck;

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

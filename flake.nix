{
  description = "Lattice — graph-native workflow automation built with Rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    rust-overlay = {
      url = "github:oxalica/rust-overlay/3c38e1e1ba9c8d7030f7b5a801398ea7d8a6fdc0";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    octet.url = "github:OnixResearch/octet/bac0f3220ada38110c03498ccf2beae5ee83e642";
    bounded-exec = {
      url = "git+https://git.onix.computer/z2CpqLFpdP36fZXYUK5ZNWxMibpCo.git?rev=29dac88ecded94457572db3fdfaaaab95fa91525";
      flake = false;
    };
    choregraph = {
      url = "git+https://seed.radicle.garden/zL2ncTUeASVYwcoGkEXv9JKgGbAF.git?rev=0b2000edaf86d5a7a8601246afbc0d526f4a2a07";
      flake = false;
    };
    cap-root = {
      url = "git+https://seed.radicle.garden/z3cWBCK7VPbEmbL5MAvKJWKjCE745.git?rev=545e77e5a49e4e1d34037aa9a82d924944ac0a1b";
      flake = false;
    };
    nickel-eval = {
      url = "git+https://seed.radicle.garden/z21miuQLqLwe1HkaMu2mEZy5fdG9F.git?rev=bf16cf223ed3dfe75a28f02ac21ed7891a60bac1";
      flake = false;
    };
    content-identity-core = {
      url = "git+https://seed.radicle.garden/zA14jhTdwST4PxPBdsryc4zS7Kpq.git?rev=7f55597b5dc879b7601856e8d7fd0dbacaa2a498";
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
      choregraph,
      nickel-eval,
      content-identity-core,
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

        workflowContractRoot = ./crates/lattice-workflow-contract;
        workflowContractToolchain = rustToolchain.override {
          targets = [ "wasm32-unknown-unknown" ];
        };
        workflowContractCraneLib = (crane.mkLib pkgs).overrideToolchain workflowContractToolchain;
        workflowContractSource = src;
        workflowContractVendorDir = cargoVendorDir;
        workflowContractCommonArgs = {
          pname = "lattice-workflow-contract";
          version = "0.1.0";
          src = workflowContractSource;
          cargoLock = ./Cargo.lock;
          cargoVendorDir = workflowContractVendorDir;
          nativeBuildInputs = with pkgs; [
            clang
            mold
          ];
          strictDeps = true;
        };
        workflowContractCargoArtifacts = workflowContractCraneLib.buildDepsOnly (
          workflowContractCommonArgs
          // {
            cargoExtraArgs = "--package lattice-workflow-contract --all-features";
          }
        );
        workflowContractOctetConfig =
          (builtins.fromJSON (builtins.readFile (workflowContractRoot + "/config/octet-deny-all.json")))
          .config;
        workflowContractOctetFlags = builtins.concatStringsSep " " (
          map (lint: "-D ${lint}") workflowContractOctetConfig.lints
        );
        workflowContractOctetRunner = pkgs.writeShellApplication {
          name = "lattice-workflow-contract-octet-deny-all";
          runtimeInputs = [
            octet.packages.${system}.cargo-octet
            pkgs.jq
          ];
          text = ''
            if [ ! -f crates/lattice-workflow-contract/Cargo.toml ]; then
              echo "run the workflow contract Octet hook from the Lattice repository root" >&2
              exit 1
            fi
            artifact_dir="$(mktemp -d)"
            trap 'rm -rf "$artifact_dir"' EXIT
            cargo-octet check -p lattice-workflow-contract --output-format json --artifact-dir "$artifact_dir" "$@"
            jq --exit-status '
              .status == "clean"
              and (.total_findings | tonumber) == 0
              and .cargo_process_exit.classification == "success"
              and (.cargo_process_exit.code | tonumber) == 0
            ' "$artifact_dir/status.json" > /dev/null
          '';
        };
        workflowContractOctetCheck = octet.lib.mkConsumerCheck {
          inherit system;
          src = workflowContractSource;
          cargoLock = ./Cargo.lock;
          packages = [ "lattice-workflow-contract" ];
        };

        workflowTransportRoot = workflowContractRoot + "/transport";
        workflowTransportSource = src;
        workflowTransportVendorDir = cargoVendorDir;
        workflowTransportCargoExtraArgs = "--package lattice-workflow-transport --all-features";
        workflowTransportCommonArgs = {
          pname = "lattice-workflow-transport";
          version = "0.1.0";
          src = workflowTransportSource;
          cargoLock = ./Cargo.lock;
          cargoVendorDir = workflowTransportVendorDir;
          nativeBuildInputs = with pkgs; [
            clang
            mold
          ];
          strictDeps = true;
        };
        workflowTransportCargoArtifacts = workflowContractCraneLib.buildDepsOnly (
          workflowTransportCommonArgs
          // {
            cargoExtraArgs = workflowTransportCargoExtraArgs;
          }
        );
        workflowTransportOctetConfig =
          (builtins.fromJSON (builtins.readFile (workflowTransportRoot + "/config/octet-deny-all.json")))
          .config;
        workflowTransportOctetFlags = builtins.concatStringsSep " " (
          map (lint: "-D ${lint}") workflowTransportOctetConfig.lints
        );
        workflowTransportOctetRunner = pkgs.writeShellApplication {
          name = "lattice-workflow-transport-octet-deny-all";
          runtimeInputs = [
            octet.packages.${system}.cargo-octet
            pkgs.jq
          ];
          text = ''
            if [ ! -f crates/lattice-workflow-contract/transport/Cargo.toml ]; then
              echo "run the workflow transport Octet hook from the Lattice repository root" >&2
              exit 1
            fi
            artifact_dir="$(mktemp -d)"
            trap 'rm -rf "$artifact_dir"' EXIT
            cargo-octet check -p lattice-workflow-transport --output-format json --artifact-dir "$artifact_dir" "$@"
            jq --exit-status '
              .status == "clean"
              and (.total_findings | tonumber) == 0
              and .cargo_process_exit.classification == "success"
              and (.cargo_process_exit.code | tonumber) == 0
            ' "$artifact_dir/status.json" > /dev/null
          '';
        };
        workflowTransportOctetCheck = octet.lib.mkConsumerCheck {
          inherit system;
          src = workflowTransportSource;
          cargoLock = ./Cargo.lock;
          packages = [ "lattice-workflow-transport" ];
        };

        # Common source filtering
        sourceRoot = toString ./.;
        isSourceFixturePath =
          path:
          pkgs.lib.hasPrefix "${sourceRoot}/examples/" (toString path)
          || pkgs.lib.hasPrefix "${sourceRoot}/tests/" (toString path)
          || pkgs.lib.hasPrefix "${sourceRoot}/.cairn/specs/" (toString path)
          || pkgs.lib.hasPrefix "${sourceRoot}/cairn-policy/" (toString path);
        isAnimusCompileTimeAssetPath =
          path:
          pkgs.lib.hasPrefix "${sourceRoot}/vendor-cargo-deps/animus/config/" (toString path)
          || pkgs.lib.hasPrefix "${sourceRoot}/vendor-cargo-deps/animus/policy/" (toString path);
        isRatsCompileTimeAssetPath =
          path:
          toString path == "${sourceRoot}/vendor-cargo-deps/rats/source/SOURCE"
          || pkgs.lib.hasPrefix "${sourceRoot}/vendor-cargo-deps/rats/source/ratcore/src/generated/" (
            toString path
          )
          || pkgs.lib.hasPrefix "${sourceRoot}/vendor-cargo-deps/rats/source/docs/generated/" (toString path)
          || pkgs.lib.hasPrefix "${sourceRoot}/vendor-cargo-deps/rats/source/meteorite/crates/met-core/generated/" (
            toString path
          )
          || pkgs.lib.hasPrefix "${sourceRoot}/vendor-cargo-deps/rats/source/themes/" (toString path);
        isPolicyAssetPath =
          path:
          pkgs.lib.hasPrefix "${sourceRoot}/policy/" (toString path)
          || pkgs.lib.hasPrefix "${sourceRoot}/contracts/" (toString path)
          || toString path == "${sourceRoot}/vendor-cargo-deps/basalt/contracts/generated/policy.json"
          || toString path == "${sourceRoot}/vendor-cargo-deps/basalt/SOURCE"
          || toString path == "${sourceRoot}/vendor-cargo-deps/basalt/NORMALIZATION.md";
        isWorkflowTemplateAssetPath =
          path: pkgs.lib.hasPrefix "${sourceRoot}/builtins/workflow-templates/" (toString path);
        isAgentEvaluationAssetPath =
          path: pkgs.lib.hasPrefix "${sourceRoot}/builtins/agent-evaluations/" (toString path);
        isOperationAssetPath =
          path: pkgs.lib.hasPrefix "${sourceRoot}/builtins/operations/" (toString path);
        isRepositoryValidationAssetPath = path: toString path == "${sourceRoot}/.gitignore";
        isWorkflowContractAssetPath =
          path:
          pkgs.lib.hasInfix "/crates/lattice-workflow-contract/" (toString path)
          && (
            pkgs.lib.hasInfix "/fixtures/" (toString path)
            || builtins.baseNameOf path == "LICENSE"
            || builtins.baseNameOf path == "rustfmt.toml"
          );
        isIdentityAuditAssetPath =
          path:
          toString path == "${sourceRoot}/README.md"
          || toString path == "${sourceRoot}/flake.nix"
          || toString path == "${sourceRoot}/flake.lock"
          || pkgs.lib.hasPrefix "${sourceRoot}/scripts/" (toString path)
          || pkgs.lib.hasPrefix "${sourceRoot}/.cairn/archive/2026-07-20-rename-n8nclone-to-lattice/" (
            toString path
          );
        src = pkgs.lib.cleanSourceWith {
          src = ./.;
          filter =
            path: type:
            !pkgs.lib.hasPrefix "${sourceRoot}/target/" (toString path)
            && (
              craneLib.filterCargoSources path type
              || isSourceFixturePath path
              || isAnimusCompileTimeAssetPath path
              || isRatsCompileTimeAssetPath path
              || isPolicyAssetPath path
              || isWorkflowTemplateAssetPath path
              || isAgentEvaluationAssetPath path
              || isOperationAssetPath path
              || isRepositoryValidationAssetPath path
              || isWorkflowContractAssetPath path
              || isIdentityAuditAssetPath path
            );
        };

        boundedExecCargoSource = "git+https://git.onix.computer/z2CpqLFpdP36fZXYUK5ZNWxMibpCo.git?rev=29dac88ecded94457572db3fdfaaaab95fa91525#29dac88ecded94457572db3fdfaaaab95fa91525";
        choregraphCargoSource = "git+https://seed.radicle.garden/zL2ncTUeASVYwcoGkEXv9JKgGbAF.git?rev=0b2000edaf86d5a7a8601246afbc0d526f4a2a07#0b2000edaf86d5a7a8601246afbc0d526f4a2a07";
        nickelEvalCargoSource = "git+https://seed.radicle.garden/z21miuQLqLwe1HkaMu2mEZy5fdG9F.git?rev=bf16cf223ed3dfe75a28f02ac21ed7891a60bac1#bf16cf223ed3dfe75a28f02ac21ed7891a60bac1";
        contentIdentityCoreCargoSource = "git+https://seed.radicle.garden/zA14jhTdwST4PxPBdsryc4zS7Kpq.git?rev=7f55597b5dc879b7601856e8d7fd0dbacaa2a498#7f55597b5dc879b7601856e8d7fd0dbacaa2a498";
        baseCargoVendorDir = craneLib.vendorCargoDeps {
          inherit src;
          overrideVendorGitCheckout =
            packages: checkout:
            if builtins.all (package: package.source == boundedExecCargoSource) packages then
              checkout.overrideAttrs (_old: {
                src = bounded-exec;
              })
            else if builtins.all (package: package.source == choregraphCargoSource) packages then
              checkout.overrideAttrs (_old: {
                src = choregraph;
              })
            else if builtins.all (package: package.source == nickelEvalCargoSource) packages then
              checkout.overrideAttrs (_old: {
                src = nickel-eval;
              })
            else if builtins.all (package: package.source == contentIdentityCoreCargoSource) packages then
              checkout.overrideAttrs (_old: {
                src = content-identity-core;
              })
            else
              checkout;
        };
        cargoVendorDir = pkgs.runCommand "lattice-cargo-vendor" { } ''
          cp -RL ${baseCargoVendorDir}/. "$out"
          chmod -R u+w "$out"
          substituteInPlace "$out/config.toml" \
            --replace-fail "${baseCargoVendorDir}" "$out"

          choregraphVendorRoot=$(find "$out" -type d -name choregraph-choreography-0.1.0 -print -quit)
          test -n "$choregraphVendorRoot"
          choregraphCheckoutRoot=$(dirname "$(dirname "$choregraphVendorRoot")")

          # Choregraph keeps reviewed compile-time inputs above package roots.
          # Restore those exact pinned files where rustc resolves each relative path.
          install -D -m 0444 \
            ${choregraph}/vendor/kamacite-core-selected/preserves/codec.rs \
            "$out/vendor/kamacite-core-selected/preserves/codec.rs"
          mkdir -p "$choregraphCheckoutRoot/schemas"
          cp -RL ${choregraph}/schemas/preserves \
            "$choregraphCheckoutRoot/schemas/preserves"
        '';

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

            run_check() {
              local artifact_dir="$1"
              local cargo_target_dir="$2"
              local baseline_file="$3"
              local status_file="$artifact_dir/status.json"
              local summary_file="$artifact_dir/summary.txt"
              local octet_status="$success_status"
              shift 3

              rm -rf "$artifact_dir"
              mkdir -p "$artifact_dir"

              local -a baseline_args=()
              if [ -f "$baseline_file" ]; then
                baseline_args=(--baseline "$baseline_file")
              fi

              set +e
              CARGO_TARGET_DIR="$cargo_target_dir" cargo-octet check "$@" --output-format json --artifact-dir "$artifact_dir" "''${baseline_args[@]}" > "$artifact_dir/stdout.json"
              octet_status="$?"
              set -e

              if [ ! -f "$status_file" ]; then
                return "$octet_status"
              fi

              # r[impl lattice.repository_validation.baselined_octet_gate]
              # r[impl lattice.repository_validation.baselined_octet_gate.scenario.accepted]
              if jq --exit-status '
                (.cargo_process_exit.classification == "success")
                and (.cargo_process_exit.code == 0)
                and (
                  (
                    .status == "clean"
                    and (
                      .baseline == null
                      or (.baseline.new_total | tonumber) == 0
                    )
                  )
                  or (
                    .status == "warning-only"
                    and .baseline != null
                    and (.baseline.new_total | tonumber) == 0
                    and (.baseline.stale_total | tonumber) == 0
                    and (.baseline.current_total | tonumber) > 0
                    and (.baseline.accepted_total | tonumber) == (.baseline.current_total | tonumber)
                    and (.baseline.accepted_current | tonumber) == (.baseline.current_total | tonumber)
                  )
                )
              ' "$status_file" > /dev/null; then
                return "$success_status"
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
              return "$lint_failure_status"
            }

            run_check \
              "''${OCTET_ARTIFACT_DIR:-target/octet-precommit}" \
              "''${OCTET_CARGO_TARGET_DIR:-target/octet-cargo-target}" \
              "''${OCTET_BASELINE:-octet-baseline.json}"
            run_check \
              "''${OCTET_CORE_ARTIFACT_DIR:-target/octet-core-precommit}" \
              "''${OCTET_CORE_CARGO_TARGET_DIR:-target/octet-core-cargo-target}" \
              "''${OCTET_CORE_BASELINE:-crates/lattice-core/octet-baseline.json}" \
              -p lattice-core
          '';
        };

        octetVendorDir = cargoVendorDir;

        octetCheck =
          pkgs.runCommand "octet-consumer-check"
            {
              nativeBuildInputs = nativeBuildInputs ++ [
                octetPreCommitCheck
                pkgs.jq
              ];
              inherit buildInputs;
            }
            ''
              export HOME="$PWD/home"
              mkdir -p "$HOME"
              cp -r ${self} source
              chmod -R u+w source
              cd source

              export CARGO_HOME="$PWD/.cargo-home"
              octet_vendor_root="$PWD/../octet-cargo-vendor"
              mkdir -p "$CARGO_HOME" "$octet_vendor_root"
              cp -RL "${octetVendorDir}/." "$octet_vendor_root"
              chmod -R u+w "$octet_vendor_root"
              choregraph_vendor_root=$(find "$octet_vendor_root" -type d -name choregraph-choreography-0.1.0 -print -quit)
              test -n "$choregraph_vendor_root"
              mkdir -p "$(dirname "$(dirname "$choregraph_vendor_root")")/schemas"
              cp -RL ${choregraph}/schemas/preserves \
                "$(dirname "$(dirname "$choregraph_vendor_root")")/schemas/preserves"
              substituteInPlace "$octet_vendor_root/config.toml" \
                --replace-fail "${octetVendorDir}" "$octet_vendor_root"
              cp "$octet_vendor_root/config.toml" "$CARGO_HOME/config.toml"

              # r[verify lattice.repository_validation.baselined_octet_gate.scenario.accepted]
              OCTET_ARTIFACT_DIR="$PWD/target/octet-check" octet-precommit-check

              positive_root="$PWD"
              negative_root="$positive_root/../negative-source"
              cp -r ${self} "$negative_root"
              chmod -R u+w "$negative_root"
              printf '\npub fn octet_negative_fixture() { todo!("negative fixture"); }\n' >> "$negative_root/src/lib.rs"

              # r[verify lattice.repository_validation.baselined_octet_gate.scenario.new_finding]
              expected_rejection_status=1
              set +e
              (
                cd "$negative_root"
                OCTET_ARTIFACT_DIR="$negative_root/target/octet-negative" \
                  OCTET_CARGO_TARGET_DIR="$negative_root/target/octet-negative-cargo" \
                  octet-precommit-check > "$negative_root/negative.log" 2>&1
              )
              negative_status="$?"
              set -e
              if [ "$negative_status" -ne "$expected_rejection_status" ]; then
                cat "$negative_root/negative.log" >&2
                echo "Octet negative fixture returned $negative_status; expected $expected_rejection_status" >&2
                exit "$expected_rejection_status"
              fi
              jq --exit-status '
                (.cargo_process_exit.classification == "success")
                and (.cargo_process_exit.code == 0)
                and (.baseline != null)
                and (.baseline.new_total | tonumber) > 0
              ' "$negative_root/target/octet-negative/status.json" > /dev/null

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

        # r[verify lattice.choregraph_adoption.source]
        # r[verify lattice.choregraph_adoption.source_alignment]
        # r[verify lattice.choregraph_adoption.single_implementation]
        choregraphSourceIdentityCheck =
          pkgs.runCommand "lattice-choregraph-source-identity"
            {
              nativeBuildInputs = [
                pkgs.coreutils
                pkgs.gnugrep
                pkgs.jq
                pkgs.nickel
              ];
            }
            ''
              set -eu
              admission=${self}/tests/fixtures/radicle/choregraph-source-admission-v1.ncl
              nickel export --format json "$admission" > "$TMPDIR/admission.json"
              jq --exit-status '
                .admission.schema == "lattice.choregraph-source-admission.v1"
                and .admission.status == "accepted"
                and .admission.rid == "rad:zL2ncTUeASVYwcoGkEXv9JKgGbAF"
                and .admission.revision == "0b2000edaf86d5a7a8601246afbc0d526f4a2a07"
                and .admission.cargo_revision == .admission.revision
                and .admission.nix_revision == .admission.revision
                and .admission.source_archive_blake3 == "8c35370153e249cc2fe9454746a82950d830e1451b3cd77cece96321acf1ea1e"
                and .admission.release_bundle_blake3 == "a2ac1336679542fa9573b356daddc18266fdda8147ddbe28f85aa37d135030d9"
                and .admission.candidate_blake3 == "e4b5cd7bd77d78c20cf8b96cd0c038144028d2287000757c8ab68f5f5dfdff1a"
                and ([.admission.prerequisites[]] | all(. == true))
                and ([.admission.evidence[]] | all(. == true))
                and .admission.forbidden_fallbacks == ["floating", "github", "sibling", "vendor", "runtime"]
                and (.admission.non_claims | length) == 5
              ' "$TMPDIR/admission.json" > /dev/null
              if nickel export --format json \
                ${self}/tests/fixtures/radicle/negative/choregraph-floating-source.ncl \
                > "$TMPDIR/invalid.json"; then
                echo 'floating Choregraph source unexpectedly passed' >&2
                exit 1
              fi

              grep --fixed-strings 'package = "choregraph-seaglass"' ${self}/Cargo.toml > /dev/null
              grep --fixed-strings 'git = "https://seed.radicle.garden/zL2ncTUeASVYwcoGkEXv9JKgGbAF.git"' ${self}/Cargo.toml > /dev/null
              grep --fixed-strings 'rev = "0b2000edaf86d5a7a8601246afbc0d526f4a2a07"' ${self}/Cargo.toml > /dev/null
              test "$(grep --count '^name = "choregraph-choreography"$' ${self}/Cargo.lock)" -eq 1
              test "$(grep --count '^name = "choregraph-core"$' ${self}/Cargo.lock)" -eq 1
              test "$(grep --count '^name = "choregraph-nickel"$' ${self}/Cargo.lock)" -eq 1
              test "$(grep --count '^name = "choregraph-seaglass"$' ${self}/Cargo.lock)" -eq 1
              grep --fixed-strings '${choregraphCargoSource}' ${self}/Cargo.lock > /dev/null
              jq --exit-status '
                .nodes.choregraph.locked.rev == "0b2000edaf86d5a7a8601246afbc0d526f4a2a07"
                and .nodes.choregraph.original.rev == "0b2000edaf86d5a7a8601246afbc0d526f4a2a07"
              ' ${self}/flake.lock > /dev/null

              test -f ${choregraph}/crates/choregraph-choreography/Cargo.toml
              test -f ${choregraph}/crates/choregraph-core/Cargo.toml
              test -f ${choregraph}/crates/choregraph-nickel/Cargo.toml
              test -f ${choregraph}/crates/choregraph-seaglass/Cargo.toml
              grep --fixed-strings 'GNU AFFERO GENERAL PUBLIC LICENSE' ${choregraph}/LICENSE > /dev/null
              test ! -e ${self}/vendor-cargo-deps/seaglass-execution-graph
              grep -R -l 'seaglass_execution_graph' ${src}/src --include='*.rs' | sort > "$TMPDIR/seaglass-import-files"
              printf '%s\n%s\n' \
                "${src}/src/engine/runner.rs" \
                "${src}/src/engine/effect/choregraph.rs" \
                | sort > "$TMPDIR/expected-seaglass-import-files"
              cmp "$TMPDIR/expected-seaglass-import-files" "$TMPDIR/seaglass-import-files"
              grep -R -l 'choregraph_nickel' ${self}/src --include='*.rs' | sort > "$TMPDIR/choregraph-import-files"
              printf '%s\n%s\n' \
                "${self}/src/workflow_format.rs" \
                "${self}/src/workflow_format/choregraph.rs" \
                > "$TMPDIR/expected-choregraph-import-files"
              cmp "$TMPDIR/expected-choregraph-import-files" "$TMPDIR/choregraph-import-files"
              grep --fixed-strings 'choregraph::lower_portable(&portable)?;' \
                ${self}/src/workflow_format.rs > /dev/null
              if grep -E 'std::fs|std::process|std::net|std::env|Path|Command|read_to_string|read_dir|tokio::|async|unsafe' \
                ${self}/src/workflow_format/choregraph.rs; then
                echo 'Choregraph lowering core contains forbidden shell authority' >&2
                exit 1
              fi
              if grep -E 'github[.]com.*choregraph|path[[:space:]]*=.*seaglass-execution-graph|vendor-cargo-deps/seaglass-execution-graph' \
                ${self}/Cargo.toml; then
                echo 'forbidden Choregraph fallback remains' >&2
                exit 1
              fi
              touch "$out"
            '';

        contentIdentityCoreSourceCheck =
          pkgs.runCommand "lattice-content-identity-core-source"
            {
              nativeBuildInputs = [ pkgs.jq ];
            }
            ''
              revision=7f55597b5dc879b7601856e8d7fd0dbacaa2a498
              source_url=https://seed.radicle.garden/zA14jhTdwST4PxPBdsryc4zS7Kpq.git
              test -f ${content-identity-core}/Cargo.toml
              test -f ${content-identity-core}/LICENSE
              grep --fixed-strings 'name = "content-identity-core"' ${content-identity-core}/Cargo.toml > /dev/null
              grep --fixed-strings 'license = "MIT"' ${content-identity-core}/Cargo.toml > /dev/null
              grep --fixed-strings "content-identity-core = { git = \"$source_url\", rev = \"$revision\" }" ${self}/Cargo.toml > /dev/null
              grep --fixed-strings "git+$source_url?rev=$revision#$revision" ${self}/Cargo.lock > /dev/null
              jq --exit-status \
                --arg revision "$revision" \
                --arg source_url "$source_url" \
                '.nodes["content-identity-core"].locked.rev == $revision
                  and .nodes["content-identity-core"].locked.url == $source_url
                  and .nodes["content-identity-core"].original.rev == $revision' \
                ${self}/flake.lock > /dev/null
              if grep -R --fixed-strings 'github.com/OnixResearch/content-identity-core' \
                ${self}/Cargo.toml ${self}/Cargo.lock ${self}/flake.lock; then
                echo 'forbidden content-identity-core GitHub fallback' >&2
                exit 1
              fi
              if grep -E 'content_identity_core|crate::content_identity' \
                ${self}/src/credentials.rs ${self}/src/triggers.rs; then
                echo 'security owner imported shared content identity' >&2
                exit 1
              fi
              grep --fixed-strings 'content_identity::schedule_occurrence' \
                ${self}/src/scheduler/admission.rs > /dev/null
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

        agentEvaluationCatalogCheck =
          pkgs.runCommand "lattice-agent-evaluation-catalog"
            {
              nativeBuildInputs = [ pkgs.nickel ];
            }
            ''
              nickel typecheck ${src}/policy/agent-evaluation-catalog.ncl
              nickel export --format json ${src}/builtins/agent-evaluations/catalog.ncl > catalog.json
              nickel typecheck ${src}/builtins/workflow-templates/agent-evaluation-catalog-review/manifest.ncl
              nickel typecheck ${src}/builtins/workflow-templates/agent-evaluation-catalog-review/workflow.ncl
              touch "$out"
            '';

        operationContractCheck =
          pkgs.runCommand "lattice-operation-contracts"
            {
              nativeBuildInputs = [ pkgs.nickel ];
            }
            ''
              nickel typecheck ${src}/policy/operation-contracts.ncl
              unexpected_success=1
              nickel export --format json ${src}/builtins/operations/catalog.ncl > catalog.json
              nickel export --format json ${src}/builtins/operations/instances.example.ncl > instances.json
              if nickel export --format json ${src}/tests/fixtures/operations/catalog-unknown-field.ncl > /dev/null 2>&1; then
                echo "invalid operation catalog unexpectedly exported" >&2
                exit "$unexpected_success"
              fi
              touch "$out"
            '';

        operationDataPolicyCheck =
          pkgs.runCommand "lattice-operation-data-policy"
            {
              nativeBuildInputs = [ pkgs.nickel ];
            }
            ''
              unexpected_success=1
              nickel typecheck ${src}/policy/operation-data.ncl
              nickel export --format json ${src}/tests/fixtures/operation-data/query-valid.ncl > query.json
              if nickel export --format json ${src}/tests/fixtures/operation-data/query-invalid-identity.ncl > /dev/null 2>&1; then
                echo "invalid operation-data query unexpectedly exported" >&2
                exit "$unexpected_success"
              fi
              touch "$out"
            '';

        operationPackPolicyCheck =
          pkgs.runCommand "lattice-operation-pack-policy"
            {
              nativeBuildInputs = [ pkgs.nickel ];
            }
            ''
              unexpected_success=1
              nickel typecheck ${src}/policy/operation-packs.ncl
              nickel export --format json ${src}/tests/fixtures/operation-packs/manifest-policy-valid.ncl > manifest.json
              nickel export --format json ${src}/tests/fixtures/operation-packs/registry-valid.ncl > registry.json
              if nickel export --format json ${src}/tests/fixtures/operation-packs/manifest-policy-invalid.ncl > /dev/null 2>&1; then
                echo "invalid operation-pack manifest unexpectedly exported" >&2
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
          workflow-contract-octet-deny-all = workflowContractOctetRunner;
          workflow-transport-octet-deny-all = workflowTransportOctetRunner;
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

          agent-evaluation = craneLib.cargoTest {
            inherit
              src
              cargoArtifacts
              cargoVendorDir
              nativeBuildInputs
              buildInputs
              ;
            cargoTestExtraArgs = "--test test_agent_evaluation_core --test test_agent_evaluation_shell";
          };

          operation-contracts = craneLib.cargoTest {
            inherit
              src
              cargoArtifacts
              cargoVendorDir
              nativeBuildInputs
              buildInputs
              ;
            cargoTestExtraArgs = "--test test_operation_contracts --test test_operation_cli";
          };

          operation-data = craneLib.cargoTest {
            inherit
              src
              cargoArtifacts
              cargoVendorDir
              nativeBuildInputs
              buildInputs
              ;
            cargoTestExtraArgs = "--test test_operation_data";
          };

          operation-packs = craneLib.cargoTest {
            inherit
              src
              cargoArtifacts
              cargoVendorDir
              nativeBuildInputs
              buildInputs
              ;
            cargoTestExtraArgs = "--test test_operation_pack_core --test test_operation_pack_analyzers --test test_operation_pack_shell --test test_operation_pack_cli";
          };

          workflow-contract = workflowContractCraneLib.cargoTest (
            workflowContractCommonArgs
            // {
              cargoArtifacts = workflowContractCargoArtifacts;
              cargoTestExtraArgs = "--package lattice-workflow-contract --all-targets --all-features";
            }
          );

          workflow-transport = workflowContractCraneLib.cargoTest (
            workflowTransportCommonArgs
            // {
              cargoArtifacts = workflowTransportCargoArtifacts;
              cargoTestExtraArgs = "${workflowTransportCargoExtraArgs} --all-targets";
            }
          );

          workflow-contract-wasm = workflowContractCraneLib.cargoBuild (
            workflowContractCommonArgs
            // {
              cargoArtifacts = workflowContractCargoArtifacts;
              cargoExtraArgs = "--package lattice-workflow-contract --target wasm32-unknown-unknown";
            }
          );

          workflow-contract-nickel =
            pkgs.runCommand "lattice-workflow-contract-nickel"
              {
                nativeBuildInputs = [ pkgs.nickel ];
              }
              ''
                nickel typecheck ${self}/contracts/kiln-workflow-contract.ncl
                nickel export --format json \
                  ${self}/tests/fixtures/kiln-workflow-contract/request-valid.ncl \
                  > /dev/null
                if nickel export --format json \
                  ${self}/tests/fixtures/kiln-workflow-contract/negative/request-invalid-identity.ncl \
                  > /dev/null; then
                  echo "invalid workflow identity fixture passed" >&2
                  exit 1
                fi
                if nickel export --format json \
                  ${self}/tests/fixtures/kiln-workflow-contract/negative/profile-zero-limit.ncl \
                  > /dev/null; then
                  echo "zero workflow limit fixture passed" >&2
                  exit 1
                fi
                nickel typecheck ${self}/contracts/workflow-transport-profile.ncl
                nickel export --format json \
                  ${self}/tests/fixtures/workflow-transport/profile-valid.ncl \
                  > /dev/null
                if nickel export --format json \
                  ${self}/tests/fixtures/workflow-transport/negative/profile-zero-frame.ncl \
                  > /dev/null; then
                  echo "zero transport frame fixture passed" >&2
                  exit 1
                fi
                if nickel export --format json \
                  ${self}/tests/fixtures/workflow-transport/negative/profile-empty-peers.ncl \
                  > /dev/null; then
                  echo "empty transport peer fixture passed" >&2
                  exit 1
                fi
                if nickel export --format json \
                  ${self}/tests/fixtures/workflow-transport/negative/profile-unsorted-peers.ncl \
                  > /dev/null; then
                  echo "unsorted transport peer fixture passed" >&2
                  exit 1
                fi
                nickel typecheck ${self}/contracts/workflow-exchange-handler-profile.ncl
                nickel export --format json \
                  ${self}/tests/fixtures/workflow-exchange-handler/profile-valid.ncl \
                  > /dev/null
                if nickel export --format json \
                  ${self}/tests/fixtures/workflow-exchange-handler/negative/profile-zero-active.ncl \
                  > /dev/null; then
                  echo "zero workflow exchange active limit fixture passed" >&2
                  exit 1
                fi
                if nickel export --format json \
                  ${self}/tests/fixtures/workflow-exchange-handler/negative/profile-empty-routes.ncl \
                  > /dev/null; then
                  echo "empty workflow exchange route fixture passed" >&2
                  exit 1
                fi
                touch "$out"
              '';

          agent-evaluation-catalog = agentEvaluationCatalogCheck;
          operation-contract-catalog = operationContractCheck;
          operation-data-policy = operationDataPolicyCheck;
          operation-pack-policy = operationPackPolicyCheck;

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

          workflow-contract-clippy = workflowContractCraneLib.cargoClippy (
            workflowContractCommonArgs
            // {
              cargoArtifacts = workflowContractCargoArtifacts;
              cargoClippyExtraArgs = "--package lattice-workflow-contract --all-targets --all-features --no-deps -- -D warnings";
            }
          );

          workflow-transport-clippy = workflowContractCraneLib.cargoClippy (
            workflowTransportCommonArgs
            // {
              cargoArtifacts = workflowTransportCargoArtifacts;
              cargoClippyExtraArgs = "${workflowTransportCargoExtraArgs} --all-targets --no-deps -- -D warnings";
            }
          );

          # Format check
          fmt = craneLib.cargoFmt {
            inherit src;
            cargoExtraArgs = "--package lattice";
          };

          workflow-contract-fmt = craneLib.cargoFmt {
            pname = "lattice-workflow-contract-format";
            src = workflowContractSource;
            cargoExtraArgs = "--package lattice-workflow-contract";
          };

          workflow-transport-fmt = workflowContractCraneLib.cargoFmt {
            pname = "lattice-workflow-transport-format";
            version = "0.1.0";
            src = workflowTransportSource;
            cargoExtraArgs = "--package lattice-workflow-transport";
          };

          # Octet Tiger Style lints
          octet = octetCheck;
          workflow-contract-octet = workflowContractOctetCheck;
          workflow-transport-octet = workflowTransportOctetCheck;

          # Exact published process-runner source identity
          bounded-exec-source = boundedExecSourceIdentityCheck;

          # Exact Choregraph source and single adapter path
          choregraph-source = choregraphSourceIdentityCheck;

          # Exact content identity source and security-role boundary
          content-identity-core-source = contentIdentityCoreSourceCheck;

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
              workflowContractOctetRunner
              workflowTransportOctetRunner
            ];

          # Use the exact toolchain required by the admitted Choregraph source
          inputsFrom = [ lattice ];
        };
      }
    );
}

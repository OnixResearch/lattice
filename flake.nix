{
  description = "Lattice — graph-native workflow automation built with Rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    nickel-1-17-0.url = "github:nickel-lang/nickel/1320a983e6c3d1e2fb53dd2464b084b4903b1426";
    crane.url = "github:ipetkov/crane";
    rust-overlay = {
      url = "github:oxalica/rust-overlay/3c38e1e1ba9c8d7030f7b5a801398ea7d8a6fdc0";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    octet.url = "github:OnixResearch/octet/bac0f3220ada38110c03498ccf2beae5ee83e642";
    cairn = {
      url = "git+ssh://git@github.com/OnixResearch/cairn.git?rev=5e1fe08d62149b286ff583eb056e655fe92e3cf1";
      flake = false;
    };
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
      url = "git+https://seed.radicle.garden/z21miuQLqLwe1HkaMu2mEZy5fdG9F.git?rev=f23be09e41337de8fa729274f50023f861ae0e8a";
      flake = false;
    };
    content-identity-core = {
      url = "git+https://seed.radicle.garden/zA14jhTdwST4PxPBdsryc4zS7Kpq.git?rev=7f55597b5dc879b7601856e8d7fd0dbacaa2a498";
      flake = false;
    };
    transactional-reconciliation-core = {
      url = "git+https://seed.radicle.garden/z4Tky6zvC8w4Y6c4YBzNxVbq5n752.git?rev=eb2bd3441753af97bfcb247cef7cc22d72675b62";
      flake = false;
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      nickel-1-17-0,
      crane,
      rust-overlay,
      flake-utils,
      octet,
      cairn,
      bounded-exec,
      choregraph,
      nickel-eval,
      content-identity-core,
      transactional-reconciliation-core,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        exactNickel = nickel-1-17-0.packages.${system}.default;
        cairnNative =
          (builtins.getFlake (builtins.unsafeDiscardStringContext "path:${cairn}?narHash=${cairn.narHash}"))
          .packages.${system}.cairn;
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            (import rust-overlay)
            (_final: _previous: { nickel = exactNickel; })
          ];
        };

        cairnCli = pkgs.writeShellApplication {
          name = "cairn";
          runtimeInputs = [ pkgs.nix ];
          text = ''
            policy_path=${pkgs.lib.escapeShellArg "${./cairn-policy/generated/cairn-policy.json}"}
            source_flake=${pkgs.lib.escapeShellArg "path:${cairn}"}

            has_policy_arg=false
            for argument in "$@"; do
              if [ "$argument" = "--policy" ]; then
                has_policy_arg=true
                break
              fi
            done

            command="''${1:-}"
            case "$command" in
              agent-receipt|change|doctor|validate|release-readiness|policy|probe|steel-orchestration|traceability|tracey|migrate|gate|sync|archive)
                if [ "$has_policy_arg" = false ]; then
                  exec nix run "$source_flake#cairn" -- "$@" --policy "$policy_path"
                fi
                ;;
            esac

            exec nix run "$source_flake#cairn" -- "$@"
          '';
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
        isToonVendorPath =
          path: pkgs.lib.hasPrefix "${sourceRoot}/src/vendor-cargo-deps/toon-format/" (toString path);
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
        isRuntimeConfigAssetPath = path: toString path == "${sourceRoot}/src/config/default.ncl";
        isCompileTimeDocPath =
          path:
          builtins.elem (toString path) (
            map (relative: "${sourceRoot}/${relative}") [
              "docs/bounded-ready-frontier.md"
              "docs/choregraph-effect-authority.md"
              "docs/content-identity-source.md"
              "docs/dag-node-memoization.md"
              "docs/expression-dependencies.md"
              "docs/nickel-eval-source.md"
              "docs/role-protocol-execution.md"
              "docs/workflow-domain-boundaries.md"
            ]
          );
        isPolicyAssetPath =
          path:
          pkgs.lib.hasPrefix "${sourceRoot}/policy/" (toString path)
          || pkgs.lib.hasPrefix "${sourceRoot}/contracts/" (toString path)
          || pkgs.lib.hasPrefix "${sourceRoot}/profiles/" (toString path)
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
              || isToonVendorPath path
              || isRatsCompileTimeAssetPath path
              || isRuntimeConfigAssetPath path
              || isCompileTimeDocPath path
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
        nickelEvalCargoSource = "git+https://seed.radicle.garden/z21miuQLqLwe1HkaMu2mEZy5fdG9F.git?rev=f23be09e41337de8fa729274f50023f861ae0e8a#f23be09e41337de8fa729274f50023f861ae0e8a";
        contentIdentityCoreCargoSource = "git+https://seed.radicle.garden/zA14jhTdwST4PxPBdsryc4zS7Kpq.git?rev=7f55597b5dc879b7601856e8d7fd0dbacaa2a498#7f55597b5dc879b7601856e8d7fd0dbacaa2a498";
        transactionalReconciliationCoreCargoSource = "git+https://seed.radicle.garden/z4Tky6zvC8w4Y6c4YBzNxVbq5n752.git?rev=eb2bd3441753af97bfcb247cef7cc22d72675b62#eb2bd3441753af97bfcb247cef7cc22d72675b62";
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
            else if
              builtins.all (package: package.source == transactionalReconciliationCoreCargoSource) packages
            then
              checkout.overrideAttrs (_old: {
                src = transactional-reconciliation-core;
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

        # r[impl lattice.repository_validation.baselined_octet_gate.scenario.boundary]
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

        # r[impl lattice.repository_validation.radicle_bounded_exec_source]
        # r[verify lattice.repository_validation.radicle_bounded_exec_source.scenario.accepted]
        # r[impl lattice.repository_validation.radicle_bounded_exec_locks]
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

              # r[impl lattice.repository_validation.radicle_bounded_exec_availability]
              # Verify the bound producer availability receipt, not a live seed-outage claim.
              admission=${self}/tests/fixtures/radicle/bounded-exec-source-admission-v1.json
              expected_hash="$(cat ${self}/tests/fixtures/radicle/bounded-exec-source-admission-v1.blake3)"
              actual_hash="$(b3sum "$admission" | cut -d ' ' -f 1)"
              test "$actual_hash" = "$expected_hash"
              nickel export --format json ${self}/tests/fixtures/radicle/bounded-exec-source-admission-v1.ncl \
                | jq -S . > "$TMPDIR/admission-from-nickel.json"
              cmp "$admission" "$TMPDIR/admission-from-nickel.json"
              # r[impl lattice.repository_validation.radicle_bounded_exec_availability.scenario.seed-outage]
              jq -e \
                --arg rid 'rad:z2CpqLFpdP36fZXYUK5ZNWxMibpCo' \
                --arg rev '29dac88ecded94457572db3fdfaaaab95fa91525' \
                --arg source_hash '4fbbf8f0749262469f00748e04c775180488dba800303f139172656d25931927' \
                '.status == "accepted" and .repository.rid == $rid and .repository.reviewed_commit == $rev and .repository.source_archive_blake3 == $source_hash and .availability.github_fallback_configured == false and .availability.seed_count == 2 and .availability.primary_only == "verified" and .availability.replica_only == "verified"' \
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

              # r[impl lattice.repository_validation.radicle_bounded_exec_source.scenario.github-rejected]
              # r[impl lattice.repository_validation.radicle_bounded_exec_availability.scenario.no-fallback]
              forbidden_source='github.com/OnixResearch/'"bounded-exec"
              if grep --fixed-strings "$forbidden_source" \
                ${self}/Cargo.toml ${self}/Cargo.lock ${self}/flake.nix ${self}/flake.lock; then
                echo 'Bounded Exec source retained a GitHub fallback' >&2
                exit 1
              fi
              # r[impl lattice.repository_validation.radicle_bounded_exec_behavior.scenario.non-claim]
              echo 'Source identity and bound availability receipt only: not Radicle protocol correctness, Bounded Exec correctness, host sandboxing, third-party GitHub independence, or release readiness.'
              touch "$out"
            '';

        # r[verify lattice.choregraph_adoption.source]
        # r[verify lattice.choregraph_adoption.source_alignment]
        # r[verify lattice.choregraph_adoption.single_implementation]
        # r[impl lattice.execution_graph_external_adoption.source]
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
              set +e
              nickel export --format json \
                ${self}/tests/fixtures/radicle/negative/choregraph-floating-source.ncl \
                > "$TMPDIR/invalid.json"
              invalid_source_status="$?"
              set -e
              if [ "$invalid_source_status" -eq 0 ]; then
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
              # r[impl lattice.execution_graph_external_adoption.cutover]
              test ! -e ${self}/vendor-cargo-deps/seaglass-execution-graph
              # r[impl lattice.execution_graph_external_adoption.authority]
              # r[impl lattice.execution_graph_external_adoption.authority.scenario.denied]
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
              grep --fixed-strings 'choregraph::lower_portable(&workflow_to_portable(&workflow))?;' \
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
              # r[impl lattice.execution_graph_external_adoption.cutover.scenario.non-claim]
              echo 'Selected graph/source checks only: not workflow correctness, scheduler fairness, effect safety, persistence correctness, Radicle availability, or release readiness.'
              touch "$out"
            '';

        # r[verify lattice.role_protocol.runtime_profile]
        # r[verify lattice.role_protocol.runtime_profile.scenario.operator]
        # r[verify lattice.role_protocol.runtime_profile.scenario.unsupported]
        # r[verify lattice.role_protocol.runtime_profile.scenario.identity]
        # r[verify lattice.role_protocol.runtime_profile.scenario.authority]
        roleProtocolRuntimeProfileCheck =
          pkgs.runCommand "lattice-role-protocol-runtime-profile"
            {
              nativeBuildInputs = [
                pkgs.b3sum
                pkgs.coreutils
                pkgs.diffutils
                pkgs.gnugrep
                pkgs.jq
                pkgs.nickel
              ];
            }
            ''
              set -eu
              contract=${self}/contracts/role-protocol-runtime-profile.ncl
              profile=${self}/profiles/role-protocol-runtime-v1.ncl
              generated=${self}/profiles/generated/role-protocol-runtime-v1.json
              generated_hash=${self}/profiles/generated/role-protocol-runtime-v1.blake3
              source_manifest=${self}/profiles/generated/lattice-role-protocol-runtime-source-v1.blake3
              fixture_root=${self}/tests/fixtures/role-protocol-runtime-profile

              nickel typecheck "$contract"
              nickel export --format json "$profile" | jq --sort-keys . > "$TMPDIR/profile.json"
              cmp "$generated" "$TMPDIR/profile.json"

              expected_profile_hash="$(cat "$generated_hash")"
              actual_profile_hash="$(b3sum "$generated" | cut -d ' ' -f 1)"
              test "$actual_profile_hash" = "$expected_profile_hash"

              source_manifest_hash="$(b3sum "$source_manifest" | cut -d ' ' -f 1)"
              test "$source_manifest_hash" = 'ef80b33109874f6d090efe4791e2980538c5901f4f137ad6d336aa9b675a6a16'
              test "$(wc -l < "$source_manifest")" -eq 24
              sort --check --key=2,2 "$source_manifest"
              test "$(grep --extended-regexp --count '^[0-9a-f]{64}  [A-Za-z0-9._/-]+$' "$source_manifest")" -eq 24
              (cd ${self} && b3sum --check "$source_manifest")
              if sed --regexp-extended 's/^[0-9a-f]{64}  //' "$source_manifest" \
                | grep --extended-regexp '(^/|(^|/)\.\.(/|$))'; then
                echo 'runtime source manifest contains an unsafe path' >&2
                exit 1
              fi

              jq --exit-status \
                --arg profile_hash "$expected_profile_hash" \
                --arg source_manifest "b3:$source_manifest_hash" '
                  .schema == "lattice.role-protocol-runtime-profile.v1"
                  and .runtime.repository == "rad:z2yGPFsQZsUcFNkiyR4Dkj22BuxnC"
                  and .runtime.revision == "cbb5db0e4844b466dcd1877218eb1f00a2a7caf2"
                  and .runtime.source_manifest_identity == $source_manifest
                  and .runtime.package == "lattice"
                  and .runtime.binary == "lattice"
                  and .runtime.command_prefix == ["--config", "__CONFIG_PATH__", "protocol"]
                  and .service.mode == "operator_oneshot"
                  and .service.start_at_boot == false
                  and .capabilities.operator_commands == ["create", "inspect", "recover", "retry", "abandon", "replay"]
                  and .capabilities.effect_dispatch == "library_only"
                  and .capabilities.cli_authority == "unavailable"
                  and .capabilities.configured_adapters == false
                  and .authority.placement == "consumer_owned"
                  and (.required_inputs | length) == 4
                  and (.non_claims | length) == 6
                  and ($profile_hash | length) == 64
                ' "$generated" > /dev/null

              for fixture in "$fixture_root"/negative/*.ncl; do
                if nickel export --format json "$fixture" > /dev/null 2>&1; then
                  echo "invalid runtime profile unexpectedly passed: $fixture" >&2
                  exit 1
                fi
              done

              protocol_cli=${self}/src/engine/protocol/cli.rs
              grep --fixed-strings 'struct UnavailableAuthority;' "$protocol_cli" > /dev/null
              grep --fixed-strings 'std::collections::HashMap::new()' "$protocol_cli" > /dev/null
              sed -n '/^pub enum CliCommand {$/,/^}$/p' "$protocol_cli" \
                | grep --extended-regexp '^    [A-Z][A-Za-z]+([[:space:]]*\{|,)' \
                | sed --regexp-extended 's/^    ([A-Za-z]+).*/\1/' \
                > "$TMPDIR/commands"
              printf '%s\n' Create Inspect Recover Retry Abandon Replay > "$TMPDIR/expected-commands"
              cmp "$TMPDIR/expected-commands" "$TMPDIR/commands"

              if grep --extended-regexp --ignore-case \
                '(-----BEGIN|/home/|/tmp/|https?://|"(password|token|private_key|endpoint)")' \
                "$profile" "$generated"; then
                echo 'runtime profile contains secret, endpoint, or host-path material' >&2
                exit 1
              fi
              touch "$out"
            '';

        # r[impl lattice.content_identity_adoption.source]
        # r[impl lattice.content_identity_adoption.source.scenario.accepted]
        # r[impl lattice.content_identity_adoption.source.scenario.rejected]
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
              # r[impl lattice.content_identity_adoption.boundary.scenario.denied]
              if grep -E 'content_identity_core|crate::content_identity' \
                ${self}/src/credentials.rs ${self}/src/triggers.rs; then
                echo 'security owner imported shared content identity' >&2
                exit 1
              fi
              grep --fixed-strings 'content_identity::schedule_occurrence' \
                ${self}/src/scheduler/admission.rs > /dev/null
              touch "$out"
            '';

        # r[impl lattice.persistence_uncertainty.shared_core]
        # r[impl lattice.persistence_uncertainty.shared_core.scenario.selected]
        transactionalReconciliationCoreSourceCheck =
          pkgs.runCommand "lattice-transactional-reconciliation-core-source"
            {
              nativeBuildInputs = [ pkgs.jq ];
            }
            ''
              revision=eb2bd3441753af97bfcb247cef7cc22d72675b62
              source_url=https://seed.radicle.garden/z4Tky6zvC8w4Y6c4YBzNxVbq5n752.git
              manifest=${transactional-reconciliation-core}/crates/transactional-reconciliation-core/Cargo.toml
              test -f "$manifest"
              test -f ${transactional-reconciliation-core}/LICENSE
              grep --fixed-strings 'name = "transactional-reconciliation-core"' "$manifest" > /dev/null
              grep --fixed-strings 'license = "MIT"' ${transactional-reconciliation-core}/Cargo.toml > /dev/null
              grep --fixed-strings "transactional-reconciliation-core = { git = \"$source_url\", rev = \"$revision\" }" ${self}/Cargo.toml > /dev/null
              grep --fixed-strings "git+$source_url?rev=$revision#$revision" ${self}/Cargo.lock > /dev/null
              jq --exit-status \
                --arg revision "$revision" \
                --arg source_url "$source_url" \
                '.nodes["transactional-reconciliation-core"].locked.rev == $revision
                  and .nodes["transactional-reconciliation-core"].locked.url == $source_url
                  and .nodes["transactional-reconciliation-core"].original.rev == $revision' \
                ${self}/flake.lock > /dev/null
              if grep -R --fixed-strings 'github.com/OnixResearch/transactional-reconciliation-core' \
                ${self}/Cargo.toml ${self}/Cargo.lock ${self}/flake.lock; then
                echo 'forbidden transactional-reconciliation-core GitHub fallback' >&2
                exit 1
              fi
              grep --fixed-strings 'transactional_reconciliation_core::admit_dispatch' \
                ${self}/src/reconciliation.rs > /dev/null
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

        structuredInputProjectionPolicyCheck =
          pkgs.runCommand "lattice-structured-input-projection-policy"
            {
              nativeBuildInputs = [
                pkgs.b3sum
                pkgs.coreutils
                pkgs.gnugrep
                pkgs.jq
                pkgs.nickel
              ];
            }
            ''
              set -eu
              fixture_root=${src}/tests/fixtures/structured-input-projection
              nickel typecheck ${src}/policy/structured-input-projection.ncl
              nickel typecheck ${src}/policy/structured-input-projection-pilot.ncl
              nickel export --format json "$fixture_root/valid.ncl" > valid.json
              nickel export --format json "$fixture_root/pilot-profile.ncl" > pilot-profile.json
              test "$(b3sum pilot-profile.json | cut -d ' ' -f 1)" = '27eee071c780b8dab66147e9accff08352665b7c69b819a753a6be5eeb297e90'
              test "$(b3sum "$fixture_root/system-prompt.txt" | cut -d ' ' -f 1)" = '75a8ec59ad2ce4381d9f153250c2317867061bc773c6a3bd0c0bb566ae386843'
              test "$(b3sum "$fixture_root/pilot-corpus.json" | cut -d ' ' -f 1)" = 'cb0a3d8ee865a076ed86d6b3a01228002928985f10d1ad5040c8816203535869'
              test "$(b3sum ${src}/src/agent_prompt/evaluation.rs | cut -d ' ' -f 1)" = '04d31e4d08149443e9119cf2d508b4b006ece476085c35a781a954f9a228914e'
              for fixture in "$fixture_root"/negative/*.ncl; do
                if nickel export --format json "$fixture" > /dev/null 2>&1; then
                  echo "invalid structured-input projection fixture passed: $fixture" >&2
                  exit 1
                fi
              done
              if grep --fixed-strings 'toon-format =' ${src}/Cargo.toml ${src}/Cargo.lock; then
                echo 'TOON must not change the accepted Lattice Cargo dependency identity' >&2
                exit 1
              fi
              grep --fixed-strings 'vendor-cargo-deps/toon-format-compile/src/encode/mod.rs' ${src}/src/lib.rs > /dev/null
              grep --fixed-strings 'vendor-cargo-deps/toon-format-compile/src/decode/mod.rs' ${src}/src/lib.rs > /dev/null
              grep --fixed-strings 'vendor-cargo-deps/toon-format-compile/src/utils/mod.rs' ${src}/src/lib.rs > /dev/null
              toon_root=${src}/src/vendor-cargo-deps/toon-format
              compile_root=${src}/src/vendor-cargo-deps/toon-format-compile
              compile_manifest_lines=$(cd "$compile_root/src" && find . -type f -name '*.rs' -printf '%P\n' | LC_ALL=C sort | while IFS= read -r derived; do printf '%s  ./%s\n' "$(b3sum "$derived" | cut -d ' ' -f 1)" "$derived"; done)
              test "$(printf '%s\n' "$compile_manifest_lines" | b3sum | cut -d ' ' -f 1)" = '2e255cece2de6905eb001898f6303699dad50422155480ad39d0f9a487a5356d' || { echo 'derived compile tree drifted from the recorded sync output' >&2; exit 1; }
              if grep --recursive --fixed-strings 'cfg(test)' "$compile_root/src"; then
                echo 'derived compile tree still carries upstream inline tests' >&2
                exit 1
              fi
              test "$(b3sum "$toon_root/Cargo.toml" | cut -d ' ' -f 1)" = '2c55674a9d444937cc5f19c5a0b5d0bf7668de47dbafb2f72c32dade1833692f'
              test "$(b3sum "$toon_root/Cargo.toml.orig" | cut -d ' ' -f 1)" = '2e93f61b34f641ac14439d2ceada6707bf01f34b8e8171ccd2edd47baec26c71'
              test "$(b3sum "$toon_root/UPSTREAM-FILES.b3" | cut -d ' ' -f 1)" = '7aef44e975bdab7a0a705bf824f5ffa0e292bd2d1cb122c6f1585364ad33dc01'
              (cd "$toon_root" && b3sum --check UPSTREAM-FILES.b3)
              if grep --fixed-strings 'features = ["preserve_order"]' "$toon_root/Cargo.toml"; then
                echo 'vendored TOON product manifest enables serde_json/preserve_order' >&2
                exit 1
              fi
              grep --fixed-strings 'features = ["preserve_order"]' "$toon_root/Cargo.toml.orig" > /dev/null
              nickel export --format json "$toon_root/SOURCE.ncl" > toon-source.json
              jq --exit-status '
                .revision == "2136cb1a35f3ed63be733bb74d36b89d3b4592dd"
                and .crates_io_sha256 == "8f89570c1a68d73941f728cca32a4345b2ffca36667ad921af336c60309a3e7e"
                and .rust_source_modified == false
                and .manifest_only_patch == true
                and .default_cli_or_tui_enabled == false
                and .serde_json_preserve_order_enabled == false
              ' toon-source.json > /dev/null
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
            buildInputs
            ;
          nativeBuildInputs = nativeBuildInputs ++ [ pkgs.nickel ];
          LATTICE_CAIRN_BIN = "${cairnNative}/bin/cairn";
        };
      in
      {
        packages = {
          default = lattice;
          cairn = cairnCli;
          lattice = lattice;
          workflow-contract-octet-deny-all = workflowContractOctetRunner;
          workflow-transport-octet-deny-all = workflowTransportOctetRunner;
        };

        checks = {
          inherit lattice;

          role-protocol-runtime-profile = roleProtocolRuntimeProfileCheck;

          # r[impl lattice.nickel_eval_adoption.source]
          # r[impl lattice.nickel_eval_adoption.source.scenario.accepted]
          # r[impl lattice.nickel_eval_adoption.source.scenario.rejected]
          # r[impl lattice.nickel_toolchain.cohort.scenario.published]
          # r[impl lattice.nickel_toolchain.cohort.scenario.absent]
          # r[impl lattice.nickel_toolchain.fixture_cohort.scenario.mixed]
          # r[impl lattice.nickel_toolchain.cohort]
          # r[impl lattice.nickel_toolchain.fixture_cohort]
          # r[verify lattice.nickel_toolchain.compatibility]
          nickel-cohort-exact =
            pkgs.runCommand "lattice-nickel-cohort-exact"
              {
                nativeBuildInputs = [
                  pkgs.gnugrep
                  pkgs.nickel
                  pkgs.coreutils
                  pkgs.diffutils
                  pkgs.jq
                  pkgs.python3
                ];
              }
              ''
                fixture_root=${self}/tests/fixtures/nickel-toolchain
                test '${nickel-1-17-0.rev}' = '1320a983e6c3d1e2fb53dd2464b084b4903b1426'
                test "$(nickel --version)" = 'nickel-lang-cli nickel 1.17.0 (rev 1320a98)'
                grep -F 'f23be09e41337de8fa729274f50023f861ae0e8a' ${self}/Cargo.toml >/dev/null
                grep -F 'nickel-lang-core = "=0.18.0"' ${self}/Cargo.toml >/dev/null
                source_url=https://seed.radicle.garden/z21miuQLqLwe1HkaMu2mEZy5fdG9F.git
                revision=f23be09e41337de8fa729274f50023f861ae0e8a
                test -f ${nickel-eval}/LICENSE
                grep -Fx 'name = "nickel-eval"' ${nickel-eval}/Cargo.toml >/dev/null
                grep -Fx 'license = "MIT"' ${nickel-eval}/Cargo.toml >/dev/null
                grep -F "nickel-eval = { git = \"$source_url\", rev = \"$revision\" }" ${self}/Cargo.toml >/dev/null
                grep -F "git+$source_url?rev=$revision#$revision" ${self}/Cargo.lock >/dev/null
                jq --exit-status --arg rev "$revision" --arg url "$source_url" '
                  .nodes["nickel-eval"].locked.rev == $rev
                  and .nodes["nickel-eval"].locked.url == $url
                  and .nodes["nickel-eval"].original.rev == $rev
                ' ${self}/flake.lock >/dev/null
                python3 -c 'import sys,tomllib; p=tomllib.load(open(sys.argv[1],"rb"))["package"]; assert {(x["name"],x["version"]) for x in p if x["name"] in ("nickel-lang","nickel-lang-core")} == {("nickel-lang","2.2.0"),("nickel-lang-core","0.18.0")}, "mixed Nickel cohort"' ${self}/Cargo.lock
                # r[impl lattice.nickel_eval_adoption.boundary]
                # r[impl lattice.nickel_eval_adoption.boundary.scenario.denied]
                grep -R -l 'nickel_eval::' ${self}/src --include='*.rs' | sort > "$TMPDIR/evaluator-imports"
                printf '%s\n' '${self}/src/nickel/mod.rs' > "$TMPDIR/expected-evaluator-imports"
                cmp "$TMPDIR/expected-evaluator-imports" "$TMPDIR/evaluator-imports"
                if grep -E 'bf16cf223ed3dfe75a28f02ac21ed7891a60bac1|nickel-lang-core = "=0.17.0"' \
                  ${self}/Cargo.toml ${self}/docs/nickel-eval-source.md; then
                  echo 'old, mixed, or fallback Nickel evaluator source remains' >&2
                  exit 1
                fi
                # r[verify lattice.nickel_toolchain.compatibility.scenario.matrix]
                nickel export --format json "$fixture_root/positive/workflow.ncl" >/dev/null
                for fixture in \
                  "$fixture_root/negative/malformed.ncl" \
                  "$fixture_root/negative/missing-import.ncl" \
                  "$fixture_root/negative/contract.ncl" \
                  "$fixture_root/negative/unknown-field.ncl" \
                  ${self}/tests/fixtures/operation-packs/manifest-policy-invalid.ncl \
                  ${self}/tests/fixtures/workflow-transport/negative/profile-zero-frame.ncl; do
                  if nickel export --format json "$fixture" >/dev/null 2>&1; then
                    echo "invalid Nickel fixture unexpectedly passed: $fixture" >&2
                    exit 1
                  fi
                done
                touch "$out"
              '';

          # r[impl lattice.nickel_toolchain.validation]
          # r[impl lattice.nickel_toolchain.validation.scenario.passed]
          # Serialize process-heavy tests so the sandbox does not exhaust child-process capacity.
          nextest = craneLib.cargoNextest {
            inherit
              src
              cargoArtifacts
              cargoVendorDir
              buildInputs
              ;
            nativeBuildInputs = nativeBuildInputs ++ [ pkgs.nickel ];
            LATTICE_CAIRN_BIN = "${cairnNative}/bin/cairn";
            cargoNextestExtraArgs = "--test-threads 1";
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

          structured-input-projection-policy = structuredInputProjectionPolicyCheck;
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

          # Exact transactional reconciliation source and adapter path
          transactional-reconciliation-core-source = transactionalReconciliationCoreSourceCheck;

          # Typed traceability inventory policy and negative fixture
          traceability-inventory-policy = traceabilityInventoryPolicyCheck;
        };

        devShells.default = craneLib.devShell {
          inherit buildInputs;
          LATTICE_CAIRN_BIN = "${cairnNative}/bin/cairn";

          packages =
            (with pkgs; [
              cargo-nextest
              cargo-watch
              nickel
              pre-commit
              rust-analyzer
            ])
            ++ [
              cairnCli
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

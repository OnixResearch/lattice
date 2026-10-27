{
  description = "Basalt UCAN contract boundary toolkit";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay/3c38e1e1ba9c8d7030f7b5a801398ea7d8a6fdc0";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    # Exact stack Nickel 1.17.0 cohort.
    nickel-1-17-0.url = "github:nickel-lang/nickel/1320a983e6c3d1e2fb53dd2464b084b4903b1426";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
      nickel-1-17-0,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        nickelCohort = nickel-1-17-0.packages.${system}.default;
        validAuthorityFixtures = [
          "positive/direct-match"
          "positive/delegated-match"
          "negative/missing-grant"
          "negative/wrong-resource"
          "negative/wrong-ability"
          "negative/stale-verification-ref"
          "negative/caveat-denied"
          "negative/replay-denied"
          "negative/missing-policy-ref"
          "negative/malformed-request-ref"
          "negative/authority-overclaim"
          "negative/missing-verification-receipt"
          "negative/verification-request-mismatch"
        ];
        malformedAuthorityFixtures = [
          "weakened-non-claims"
          "invalid-disposition"
          "incompatible-expected"
        ];
        validUcanSemanticsFixtures = [
          "positive/allow"
          "positive/capability-deny"
          "negative/over-delegation"
          "negative/wildcard-request"
          "negative/missing-caveat-policy"
          "negative/stale-proof"
          "negative/replay-denied"
        ];
        malformedUcanSemanticsFixtures = [
          "final-profile-substitution"
          "missing-operation"
          "weakened-non-claims"
          "incompatible-expected-result"
          "stale-catalog"
        ];
        validLicenseConsumptionFixtures = [
          "positive/acknowledged-agpl"
          "negative/missing-review"
          "negative/wrong-artifact-identity"
          "negative/stale-license-metadata"
          "negative/weakened-non-claims"
          "negative/wrong-license-expression"
        ];
        malformedLicenseConsumptionFixtures = [
          "unsupported-schema"
        ];
        validPolicyAuthoringFixtures = [
          "positive/valid-minimal"
        ];
        invalidPolicyAuthoringFixtures = [
          "key-id-mismatch"
          "empty-allowlist"
          "unknown-backend"
          "duplicate-entry"
          "malformed-ability"
        ];
        invalidOidcPolicyFixtures = [
          "algorithm-confusion"
          "redirect-enabled"
          "replay-disabled"
          "stale-cache"
          "token-digest-replay"
          "type-mismatch"
          "weakened-non-claims"
          "zero-key-limit"
        ];
        invalidOidcSourceFixtures = [
          "crypto-backend-substitution"
          "floating-branch"
          "github-substitution"
          "jwt-version-drift"
          "revision-drift"
          "sibling-path"
          "wrong-rid"
        ];
        validNickelToolchainFixtures = [
          "valid-contract"
          "valid-export"
          "valid-import"
          "valid-merge"
        ];
        invalidNickelToolchainFixtures = [
          "contract-violation"
          "duplicate-field"
          "malformed"
          "missing-import"
          "type-mismatch"
          "unknown-field"
        ];
      in
      {
        lib.ucanSource = ./vendor/ucan;
        packages.nickel-1-17-0 = nickelCohort;

        devShells.default = pkgs.mkShell {
          packages = [
            pkgs.b3sum
            pkgs.cargo-nextest
            nickelCohort
            pkgs.openssh
            pkgs.stdenv.cc
            rustToolchain
          ];
        };

        devShells.enforcement-compat = pkgs.mkShell {
          packages = [
            pkgs.openssh
            pkgs.stdenv.cc
            (pkgs.rust-bin.stable."1.95.0".default)
          ];
        };

        checks.nickel-cohort-exact =
          pkgs.runCommand "basalt-nickel-cohort-exact"
            {
              nativeBuildInputs = [ nickelCohort ];
            }
            ''
              test '${nickel-1-17-0.rev}' = '1320a983e6c3d1e2fb53dd2464b084b4903b1426'
              test "$(command -v nickel)" = '${nickelCohort}/bin/nickel'
              test "$(nickel --version)" = 'nickel-lang-cli nickel 1.17.0 (rev 1320a98)'
              if grep -q 'pkgs\.nickel' ${self}/flake.nix; then
                echo 'older, floating, ambient, or mixed Nickel evaluator reference found' >&2
                exit 1
              fi
              touch "$out"
            '';

        checks.nickel-toolchain-fixtures =
          pkgs.runCommand "basalt-nickel-toolchain-fixtures"
            {
              nativeBuildInputs = [ nickelCohort ];
            }
            ''
              fixture_root=${./contracts/nickel-toolchain}
              for fixture in ${builtins.concatStringsSep " " validNickelToolchainFixtures}; do
                nickel typecheck "$fixture_root/positive/$fixture.ncl"
                nickel export --format json "$fixture_root/positive/$fixture.ncl" > "$TMPDIR/$fixture.json"
                cmp "$TMPDIR/$fixture.json" "$fixture_root/generated/$fixture.json"
              done
              for fixture in ${builtins.concatStringsSep " " invalidNickelToolchainFixtures}; do
                if nickel eval "$fixture_root/negative/$fixture.ncl" > "$TMPDIR/$fixture.out" 2> "$TMPDIR/$fixture.err"; then
                  echo "negative Nickel fixture unexpectedly passed: $fixture" >&2
                  exit 1
                fi
                test -s "$TMPDIR/$fixture.err"
              done
              touch "$out"
            '';

        checks.vendored-ucan =
          pkgs.runCommand "basalt-vendored-ucan"
            {
              nativeBuildInputs = [
                pkgs.b3sum
                pkgs.stdenv.cc
                rustToolchain
              ];
            }
            ''
              cp ${./tools/sync-vendored-ucan.rs} sync-vendored-ucan.rs
              cp -R ${./tools/sync_vendored_ucan} sync_vendored_ucan
              chmod -R u+w sync-vendored-ucan.rs sync_vendored_ucan
              rustc --edition=2024 --test sync-vendored-ucan.rs -o vendored-ucan-tests
              ./vendored-ucan-tests
              rustc --edition=2024 sync-vendored-ucan.rs -o vendored-ucan-check
              checker="$PWD/vendored-ucan-check"
              cd ${./.}
              "$checker" --check
              touch $out
            '';

        checks.process-execution-policy =
          pkgs.runCommand "basalt-process-execution-policy"
            { nativeBuildInputs = [ nickelCohort ]; }
            ''
              cd ${./.}
              nickel typecheck contracts/process-execution/policy.ncl
              nickel export --format json contracts/process-execution/policy.ncl > "$TMPDIR/policy.json"
              cmp "$TMPDIR/policy.json" contracts/process-execution/policy.json
              cp "$TMPDIR/policy.json" "$out"
            '';

        checks.policy-export =
          pkgs.runCommand "basalt-policy-export"
            {
              nativeBuildInputs = [ nickelCohort ];
              processExecutionPolicy = self.checks.${system}.process-execution-policy;
            }
            ''
              mkdir -p contracts
              cp ${./contracts/policy.ncl} contracts/policy.ncl
              cp ${./contracts/policy-prelude.ncl} contracts/policy-prelude.ncl
              chmod u+w contracts/policy.ncl contracts/policy-prelude.ncl
              nickel export contracts/policy.ncl > exported-policy.json
              cmp exported-policy.json ${./contracts/generated/policy.json}
              cp exported-policy.json $out
            '';

        checks.ucan-protocol-cohort =
          pkgs.runCommand "basalt-ucan-protocol-cohort"
            {
              nativeBuildInputs = [ nickelCohort ];
            }
            ''
              cd ${./.}
              nickel typecheck contracts/ucan-profiles/cohort.ncl
              nickel export --format json contracts/ucan-profiles/cohort.ncl > "$TMPDIR/cohort.json"
              for fixture in contracts/ucan-profiles/fixtures/invalid/*.ncl; do
                if nickel export --format json "$fixture" > /dev/null 2>&1; then
                  echo "invalid UCAN protocol cohort passed: $fixture" >&2
                  exit 1
                fi
              done
              touch "$out"
            '';

        checks.ucan-v1-profile =
          pkgs.runCommand "basalt-ucan-v1-profile"
            {
              nativeBuildInputs = [
                pkgs.b3sum
                nickelCohort
              ];
            }
            ''
              cd ${./.}
              nickel typecheck contracts/ucan-profiles/final-cohort.ncl
              nickel typecheck contracts/ucan-v1-profile/source-evidence.ncl
              nickel typecheck contracts/ucan-v1-profile/mapping.ncl
              nickel export --format json contracts/ucan-v1-profile/source-evidence.ncl > "$TMPDIR/source-evidence.json"
              nickel export --format json contracts/ucan-v1-profile/mapping.ncl > "$TMPDIR/mapping.json"
              cmp "$TMPDIR/source-evidence.json" contracts/ucan-v1-profile/generated/source-evidence.json
              cmp "$TMPDIR/mapping.json" contracts/ucan-v1-profile/generated/mapping.json
              expected_revision='3a2aa043068a82663cf72ec05ed5d56a7d65ecd5'
              expected_manifest='82b5e1c414fadcbdc90ef951624c9ccd4e8eb6bdce0d57daf2830fd79cd592fb'
              test "$(tr -d '\n' < vendor/ucan-revision.txt)" = "$expected_revision"
              test "$(b3sum vendor/ucan.snapshot | cut -d' ' -f1)" = "$expected_manifest"
              for fixture in contracts/ucan-v1-profile/fixtures/invalid/*.ncl; do
                if nickel export --format json "$fixture" > /dev/null 2>"$TMPDIR/$(basename "$fixture").err"; then
                  echo "invalid final-profile fixture passed: $fixture" >&2
                  exit 1
                fi
              done
              touch "$out"
            '';

        checks.durable-authority-source =
          pkgs.runCommand "basalt-durable-authority-source"
            {
              nativeBuildInputs = [ pkgs.ripgrep ];
            }
            ''
              expected_url='https://seed.radicle.garden/z3kGDT73ja8rewvGLwJFLF5MTx1ih.git'
              expected_revision='a0de79303f3f45c18a34fff535b690a85cb3cf59'
              expected_package_count=2
              locked_source="git+$expected_url?rev=$expected_revision#$expected_revision"
              rg --fixed-strings "replay-ledger-core = { git = \"$expected_url\", rev = \"$expected_revision\", optional = true }" ${./Cargo.toml}
              rg --fixed-strings "revocation-view-core = { git = \"$expected_url\", rev = \"$expected_revision\", optional = true }" ${./Cargo.toml}
              test "$(rg --fixed-strings --count-matches "$locked_source" ${./Cargo.lock})" -eq "$expected_package_count"
              if rg --fixed-strings 'github.com/OnixResearch/durable-authority-state' ${./Cargo.toml}; then
                echo 'GitHub fallback is prohibited' >&2
                exit 1
              fi
              if rg --fixed-strings 'path = "../durable-authority-state"' ${./Cargo.toml}; then
                echo 'sibling fallback is prohibited' >&2
                exit 1
              fi
              touch "$out"
            '';

        checks.nostr-identity-policy =
          pkgs.runCommand "basalt-nostr-identity-policy"
            {
              nativeBuildInputs = [ nickelCohort ];
            }
            ''
              mkdir -p contracts/nostr-identity
              cp ${./contracts/nostr-identity/contracts.ncl} contracts/nostr-identity/contracts.ncl
              cp ${./contracts/nostr-identity/policy.ncl} contracts/nostr-identity/policy.ncl
              chmod -R u+w contracts/nostr-identity
              nickel typecheck contracts/nostr-identity/policy.ncl
              nickel export contracts/nostr-identity/policy.ncl > exported-policy.json
              cmp exported-policy.json ${./contracts/nostr-identity/generated/policy.json}
              cp exported-policy.json $out
            '';

        checks.peer-auth-policy =
          pkgs.runCommand "basalt-peer-auth-policy"
            {
              nativeBuildInputs = [ nickelCohort ];
            }
            ''
              cd ${./.}
              nickel typecheck contracts/peer-auth/policy.ncl
              nickel typecheck contracts/peer-auth/steel-fixtures.ncl
              nickel export --format json contracts/peer-auth/policy.ncl > "$TMPDIR/policy.json"
              nickel export --format json contracts/peer-auth/steel-fixtures.ncl > "$TMPDIR/steel-fixtures.json"
              cmp "$TMPDIR/policy.json" contracts/peer-auth/generated/policy.json
              cmp "$TMPDIR/steel-fixtures.json" contracts/peer-auth/generated/steel-fixtures.json
              for fixture in contracts/peer-auth/fixtures/invalid/*.ncl; do
                if nickel export --format json "$fixture" > /dev/null 2>"$TMPDIR/$(basename "$fixture").err"; then
                  echo "invalid peer-auth fixture passed: $fixture" >&2
                  exit 1
                fi
              done
              touch "$out"
            '';

        checks.runtime-effect-policy =
          pkgs.runCommand "basalt-runtime-effect-policy"
            {
              nativeBuildInputs = [ nickelCohort ];
            }
            ''
              cd ${./.}
              nickel typecheck contracts/runtime-effect/policy.ncl
              nickel typecheck contracts/runtime-effect/consumer-mappings.ncl
              nickel export --format json contracts/runtime-effect/policy.ncl > "$TMPDIR/policy.json"
              nickel export --format json contracts/runtime-effect/consumer-mappings.ncl > "$TMPDIR/consumer-mappings.json"
              cmp "$TMPDIR/policy.json" contracts/runtime-effect/generated/policy.json
              cmp "$TMPDIR/consumer-mappings.json" contracts/runtime-effect/generated/consumer-mappings.json
              for fixture in contracts/runtime-effect/fixtures/invalid/*.ncl; do
                if nickel export --format json "$fixture" > /dev/null 2>"$TMPDIR/$(basename "$fixture").err"; then
                  echo "invalid runtime-effect fixture passed: $fixture" >&2
                  exit 1
                fi
              done
              touch "$out"
            '';

        checks.world-branch-authority-policy =
          pkgs.runCommand "basalt-world-branch-authority-policy"
            {
              nativeBuildInputs = [ nickelCohort ];
            }
            ''
              cd ${./.}
              nickel typecheck contracts/world-branch-authority/policy.ncl
              nickel typecheck contracts/world-branch-authority/consumer-mappings.ncl
              nickel export --format json contracts/world-branch-authority/policy.ncl > "$TMPDIR/policy.json"
              nickel export --format json contracts/world-branch-authority/consumer-mappings.ncl > "$TMPDIR/consumer-mappings.json"
              cmp "$TMPDIR/policy.json" contracts/world-branch-authority/generated/policy.json
              cmp "$TMPDIR/consumer-mappings.json" contracts/world-branch-authority/generated/consumer-mappings.json
              for fixture in contracts/world-branch-authority/fixtures/invalid/*.ncl; do
                if nickel export --format json "$fixture" > /dev/null 2>"$TMPDIR/$(basename "$fixture").err"; then
                  echo "invalid world-branch authority fixture passed: $fixture" >&2
                  exit 1
                fi
              done
              touch "$out"
            '';

        checks.oidc-exchange-policy =
          pkgs.runCommand "basalt-oidc-exchange-policy"
            {
              nativeBuildInputs = [
                pkgs.b3sum
                nickelCohort
              ];
            }
            ''
              cd ${./.}
              nickel typecheck contracts/oidc-exchange/policy.ncl
              nickel export --format json contracts/oidc-exchange/policy.ncl > "$TMPDIR/policy.json"
              cmp "$TMPDIR/policy.json" contracts/oidc-exchange/generated/policy.json
              b3sum "$TMPDIR/policy.json" > "$TMPDIR/policy.blake3"
              for fixture in ${builtins.concatStringsSep " " invalidOidcPolicyFixtures}; do
                if nickel export --format json "contracts/oidc-exchange/fixtures/invalid/$fixture.ncl" > /dev/null 2>"$TMPDIR/$fixture.err"; then
                  echo "invalid OIDC policy fixture passed: $fixture" >&2
                  exit 1
                fi
                test -s "$TMPDIR/$fixture.err"
              done
              mkdir -p "$out"
              cp "$TMPDIR/policy.json" "$out/policy.json"
              cp "$TMPDIR/policy.blake3" "$out/policy.blake3"
            '';

        checks.oidc-exchange-sources =
          pkgs.runCommand "basalt-oidc-exchange-sources"
            {
              nativeBuildInputs = [
                nickelCohort
                pkgs.ripgrep
              ];
            }
            ''
              cd ${./.}
              expected_url='https://seed.radicle.garden/z4ARxEt93T6gJTrTZpUyKHMAemPiv.git'
              expected_revision='5abc2b93e0f288aba251a0750ce2d716eb9757fc'
              expected_jwt_revision='900010fb1be550ec2cfa8c32038eb154f9ef56d7'
              expected_jwt_checksum='881733cbc631fc9e472e24447ce32a64bedf2da498d6d8570b08edc87de71f65'
              nickel typecheck contracts/oidc-exchange/source-boundary.ncl
              nickel export --format json contracts/oidc-exchange/source-boundary.ncl > "$TMPDIR/sources.json"
              for fixture in ${builtins.concatStringsSep " " invalidOidcSourceFixtures}; do
                if nickel export --format json "contracts/oidc-exchange/fixtures/source-invalid/$fixture.ncl" > /dev/null 2>"$TMPDIR/$fixture.err"; then
                  echo "invalid OIDC source fixture passed: $fixture" >&2
                  exit 1
                fi
              done
              rg --fixed-strings "bounded-http = { git = \"$expected_url\", rev = \"$expected_revision\", optional = true }" Cargo.toml
              test "$(rg --fixed-strings --count-matches "rev=$expected_revision#$expected_revision" Cargo.lock)" -eq 2
              rg --fixed-strings 'jsonwebtoken = { version = "=11.0.0", default-features = false, features = ["rust_crypto"] }' Cargo.toml
              rg --fixed-strings "$expected_jwt_revision" contracts/oidc-exchange/source-contracts.ncl
              rg --fixed-strings "$expected_jwt_checksum" Cargo.lock
              if rg 'github.com/OnixResearch/bounded-http|path = "\.\./bounded-http"|branch = ' Cargo.toml; then
                echo 'unsafe Bounded HTTP fallback found' >&2
                exit 1
              fi
              cp "$TMPDIR/sources.json" "$out"
            '';

        checks.authority-core-fixtures =
          pkgs.runCommand "basalt-authority-core-fixtures"
            {
              nativeBuildInputs = [ nickelCohort ];
            }
            ''
              fixture_root=${./contracts/authority-core}/fixtures
              mkdir -p generated/positive generated/negative
              for fixture in ${builtins.concatStringsSep " " validAuthorityFixtures}; do
                nickel export --format json "$fixture_root/$fixture.ncl" > "generated/$fixture.json"
                cmp "generated/$fixture.json" "$fixture_root/generated/$fixture.json"
              done
              for fixture in ${builtins.concatStringsSep " " malformedAuthorityFixtures}; do
                if nickel export --format json "$fixture_root/malformed/$fixture.ncl" > /dev/null 2>"$fixture.err"; then
                  echo "malformed authority-core fixture unexpectedly exported: $fixture" >&2
                  exit 1
                fi
              done
              cp -r generated "$out"
            '';

        checks.ucan-semantics-fixtures =
          pkgs.runCommand "basalt-ucan-semantics-fixtures"
            {
              nativeBuildInputs = [ nickelCohort ];
            }
            ''
              fixture_root=${./contracts/ucan-semantics}/fixtures
              mkdir -p generated/positive generated/negative
              for fixture in ${builtins.concatStringsSep " " validUcanSemanticsFixtures}; do
                nickel export --format json "$fixture_root/$fixture.ncl" > "generated/$fixture.json"
                cmp "generated/$fixture.json" "$fixture_root/generated/$fixture.json"
              done
              for fixture in ${builtins.concatStringsSep " " malformedUcanSemanticsFixtures}; do
                if nickel export --format json "$fixture_root/malformed/$fixture.ncl" > /dev/null 2>"$fixture.err"; then
                  echo "malformed UCAN semantics fixture unexpectedly exported: $fixture" >&2
                  exit 1
                fi
              done
              cp -r generated "$out"
            '';

        checks.license-consumption-fixtures =
          pkgs.runCommand "basalt-license-consumption-fixtures"
            {
              nativeBuildInputs = [ nickelCohort ];
            }
            ''
              fixture_root=${./contracts/license-consumption}/fixtures
              mkdir -p generated/positive generated/negative
              for fixture in ${builtins.concatStringsSep " " validLicenseConsumptionFixtures}; do
                nickel export --format json "$fixture_root/$fixture.ncl" > "generated/$fixture.json"
                cmp "generated/$fixture.json" "$fixture_root/generated/$fixture.json"
              done
              for fixture in ${builtins.concatStringsSep " " malformedLicenseConsumptionFixtures}; do
                if nickel export --format json "$fixture_root/malformed/$fixture.ncl" > /dev/null 2>"$fixture.err"; then
                  echo "malformed license-consumption fixture unexpectedly exported: $fixture" >&2
                  exit 1
                fi
              done
              cp -r generated "$out"
            '';

        checks.policy-authoring-fixtures =
          pkgs.runCommand "basalt-policy-authoring-fixtures"
            {
              nativeBuildInputs = [ nickelCohort ];
            }
            ''
              mkdir -p contracts
              cp ${./contracts/policy.ncl} contracts/policy.ncl
              cp ${./contracts/policy-prelude.ncl} contracts/policy-prelude.ncl
              cp -R ${./contracts/policy-authoring} contracts/policy-authoring
              chmod -R u+w contracts
              fixture_root=contracts/policy-authoring/fixtures
              nickel export contracts/policy.ncl > exported-policy.json
              cmp exported-policy.json ${./contracts/generated/policy.json}
              mkdir -p "$out/positive"
              for fixture in ${builtins.concatStringsSep " " validPolicyAuthoringFixtures}; do
                nickel export --format json "$fixture_root/$fixture.ncl" > "$out/positive/$(basename "$fixture").json"
              done
              for fixture in ${builtins.concatStringsSep " " invalidPolicyAuthoringFixtures}; do
                if nickel export --format json "$fixture_root/negative/$fixture.ncl" > /dev/null 2>"/tmp/$fixture.err"; then
                  echo "invalid policy-authoring fixture unexpectedly exported: $fixture" >&2
                  exit 1
                fi
              done
              touch "$out/all"
            '';

        checks.downstream-compat-config =
          pkgs.runCommand "basalt-downstream-compat-config"
            {
              nativeBuildInputs = [
                pkgs.stdenv.cc
                rustToolchain
              ];
            }
            ''
              rustc --edition=2024 --test ${./tools/downstream-compat.rs} -o downstream-compat-tests
              ./downstream-compat-tests
              rustc --edition=2024 ${./tools/downstream-compat.rs} -o downstream-compat
              ./downstream-compat --basalt-root ${./.} --list
              touch $out
            '';

        checks.secretspec-issuer-fixtures =
          pkgs.runCommand "basalt-secretspec-issuer-fixtures"
            {
              nativeBuildInputs = [ pkgs.jq ];
            }
            ''
              fixture_root=${./contracts/secretspec-issuer}/fixtures
              mkdir -p "$out/positive" "$out/negative"
              for fixture in positive/issuer-key-declaration positive/rotation-overlap-window; do
                jq empty "$fixture_root/$fixture.json" >/dev/null
                cp "$fixture_root/$fixture.json" "$out/$fixture.json"
              done
              for fixture in negative/missing-required-key negative/missing-declared-key \
                  negative/wrong-scope negative/provider-failure; do
                jq empty "$fixture_root/$fixture.json" >/dev/null
                cp "$fixture_root/$fixture.json" "$out/$fixture.json"
              done
              touch "$out/all"
            '';

        checks.ast-grep-policy-audits =
          pkgs.runCommand "basalt-ast-grep-policy-audits"
            {
              nativeBuildInputs = [ pkgs.ast-grep ];
            }
            ''
              mkdir -p "$out"
              cp -R ${./tools/ast-grep} tools-ast-grep
              chmod -R u+w tools-ast-grep
              ast-grep scan --config tools-ast-grep/sgconfig.yml ${./src} > scan.txt || true
              cp scan.txt "$out/scan.txt"
              touch "$out/all"
            '';

        checks.release =
          pkgs.runCommandLocal "basalt-release-verify"
            {
              nativeBuildInputs = [
                pkgs.b3sum
                pkgs.cargo-nextest
                pkgs.git
                nickelCohort
                pkgs.openssh
                pkgs.stdenv.cc
                rustToolchain
              ];
              authorityCoreFixtures = self.checks.${system}.authority-core-fixtures;
              nickelCohortExact = self.checks.${system}.nickel-cohort-exact;
              nickelToolchainFixtures = self.checks.${system}.nickel-toolchain-fixtures;
              peerAuthPolicy = self.checks.${system}.peer-auth-policy;
              runtimeEffectPolicy = self.checks.${system}.runtime-effect-policy;
              worldBranchAuthorityPolicy = self.checks.${system}.world-branch-authority-policy;
              oidcExchangePolicy = self.checks.${system}.oidc-exchange-policy;
              oidcExchangeSources = self.checks.${system}.oidc-exchange-sources;
              policyExport = self.checks.${system}.policy-export;
              processExecutionPolicy = self.checks.${system}.process-execution-policy;
              ucanSemanticsFixtures = self.checks.${system}.ucan-semantics-fixtures;
              ucanV1Profile = self.checks.${system}.ucan-v1-profile;
              licenseConsumptionFixtures = self.checks.${system}.license-consumption-fixtures;
              secretspecIssuerFixtures = self.checks.${system}.secretspec-issuer-fixtures;
              astGrepPolicyAudits = self.checks.${system}.ast-grep-policy-audits;
            }
            ''
              export HOME="$TMPDIR/home"
              export CARGO_HOME="$TMPDIR/cargo-home"
              export RELEASE_VERIFY_RECEIPT_DIR="$out"
              export RELEASE_VERIFY_POLICY_FRESH=1
              export RELEASE_VERIFY_AUTHORITY_FRESH=1
              export RELEASE_VERIFY_SEMANTICS_FRESH=1
              export RELEASE_VERIFY_LICENSE_FRESH=1
              export RELEASE_VERIFY_ISSUER_FRESH=1
              export RELEASE_VERIFY_AST_GREP_FRESH=1
              export RELEASE_VERIFY_CARGO_STEPS=0
              mkdir -p "$HOME" "$CARGO_HOME" "$out" "$TMPDIR/release-source/tools/release_verify" "$TMPDIR/release-source/src"
              cp ${./tools/release-verify.rs} "$TMPDIR/release-source/tools/release-verify.rs"
              cp -R ${./tools/release_verify}/. "$TMPDIR/release-source/tools/release_verify/"
              chmod -R u+w "$TMPDIR/release-source/tools/release-verify.rs" "$TMPDIR/release-source/tools/release_verify"
              cp ${./src/authority_identity.rs} "$TMPDIR/release-source/src/authority_identity.rs"
              cp ${./src/ast_grep_audit_identity.rs} "$TMPDIR/release-source/src/ast_grep_audit_identity.rs"
              cp ${./src/ucan_semantics_identity.rs} "$TMPDIR/release-source/src/ucan_semantics_identity.rs"
              cp ${./src/license_consumption_identity.rs} "$TMPDIR/release-source/src/license_consumption_identity.rs"
              cp ${./src/secretspec_issuer_identity.rs} "$TMPDIR/release-source/src/secretspec_issuer_identity.rs"
              rustc --edition=2024 "$TMPDIR/release-source/tools/release-verify.rs" -o "$TMPDIR/basalt-release-verify"
              cd ${./.}
              "$TMPDIR/basalt-release-verify"
            '';
      }
    );
}

{
  description = "Basalt UCAN contract boundary toolkit";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
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
        ];        malformedLicenseConsumptionFixtures = [
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
      in
      {
        lib.ucanSource = ./vendor/ucan;

        devShells.default = pkgs.mkShell {
          packages = [
            pkgs.b3sum
            pkgs.cargo
            pkgs.cargo-nextest
            pkgs.clippy
            pkgs.nickel
            pkgs.openssh
            pkgs.rustc
            pkgs.rustfmt
            pkgs.stdenv.cc
          ];
        };

        checks.vendored-ucan =
          pkgs.runCommand "basalt-vendored-ucan"
            {
              nativeBuildInputs = [
                pkgs.b3sum
                pkgs.rustc
                pkgs.stdenv.cc
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

        checks.policy-export =
          pkgs.runCommand "basalt-policy-export"
            {
              nativeBuildInputs = [ pkgs.nickel ];
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

        checks.nostr-identity-policy =
          pkgs.runCommand "basalt-nostr-identity-policy"
            {
              nativeBuildInputs = [ pkgs.nickel ];
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

        checks.authority-core-fixtures =
          pkgs.runCommand "basalt-authority-core-fixtures"
            {
              nativeBuildInputs = [ pkgs.nickel ];
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
              nativeBuildInputs = [ pkgs.nickel ];
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
              nativeBuildInputs = [ pkgs.nickel ];
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
              nativeBuildInputs = [ pkgs.nickel ];
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
                pkgs.rustc
                pkgs.stdenv.cc
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
                pkgs.cargo
                pkgs.cargo-nextest
                pkgs.clippy
                pkgs.git
                pkgs.nickel
                pkgs.openssh
                pkgs.rustc
                pkgs.rustfmt
                pkgs.stdenv.cc
              ];
              authorityCoreFixtures = self.checks.${system}.authority-core-fixtures;
              policyExport = self.checks.${system}.policy-export;
              ucanSemanticsFixtures = self.checks.${system}.ucan-semantics-fixtures;
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

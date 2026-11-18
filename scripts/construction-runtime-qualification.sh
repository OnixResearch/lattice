#!/usr/bin/env bash
# Host-run qualification of the captured local-shell construction profile.
#
# The v3 runtime admits only root-owned immutable Nix-store executables. Nix
# build sandboxes do not map host root, so this route runs on the host from the
# store artifacts selected by the flake app. It writes retained source, the
# expanded graph, exact commands, observed results and negative controls into a
# new caller-named directory. It is bounded consumer evidence, not semantic
# equivalence, external-service, producer or release qualification.
# r[impl lattice.graph_construction.qualification]
# r[impl lattice.graph_construction.qualification.output]
set -euo pipefail

usage() {
  echo "usage: construction-runtime-qualification NEW_OUTPUT_DIRECTORY" >&2
  exit 2
}

[ "$#" -eq 1 ] || usage
for required in LATTICE_QUALIFIED_BINARY LATTICE_CLOSED_EVAL_WORKER LATTICE_CONSTRUCTION_SHELL \
  LATTICE_CONSTRUCTION_TEST_BIN LATTICE_CONSTRUCTION_EXAMPLES; do
  if [ -z "${!required:-}" ]; then
    echo "missing selected artifact: $required" >&2
    exit 2
  fi
done
case "$1" in
  *'"'* | *'\'* | *'%'* | *$'\n'*)
    echo "output directory must not contain quote, backslash, percent or newline characters" >&2
    exit 2
    ;;
esac
if [ -e "$1" ] || [ -L "$1" ]; then
  echo "refusing to reuse existing output: $1" >&2
  exit 2
fi
mkdir -- "$1"
out="$(cd -- "$1" && pwd -P)"
work="$out/work"
logs="$out/logs"
mkdir -p "$work/source" "$logs" "$out/source" "$out/expanded"

lattice="$LATTICE_QUALIFIED_BINARY"
tests="$LATTICE_CONSTRUCTION_TEST_BIN"
workflow=local-shell-demo
commands="$out/commands.tsv"
printf 'name\texpected\texit_code\tcommand\n' > "$commands"
failures=()

# record NAME pass|fail COMMAND... retains stdout/stderr and the observed exit code.
record() {
  local name="$1" expected="$2" status=0
  shift 2
  "$@" > "$logs/$name.stdout" 2> "$logs/$name.stderr" || status=$?
  printf '%s\t%s\t%s\t%s\n' "$name" "$expected" "$status" "$*" >> "$commands"
  if { [ "$expected" = pass ] && [ "$status" -ne 0 ]; } || { [ "$expected" = fail ] && [ "$status" -eq 0 ]; }; then
    failures+=("$name")
  fi
}

# expect NAME CONDITION... records a derived observation over retained outputs.
expect() {
  local name="$1" status=0
  shift
  "$@" > /dev/null 2>&1 || status=$?
  printf '%s\t%s\t%s\t%s\n' "$name" pass "$status" "$*" >> "$commands"
  [ "$status" -eq 0 ] || failures+=("$name")
}

config() {
  local path="$1" authority="${2:-}"
  {
    printf '{\n  data_dir = "%s",\n  default_timeout_secs = 60,\n  shell = "sh",\n' "$work/state"
    if [ -n "$authority" ]; then
      printf '  effect_authority = "%s",\n' "$authority"
    fi
    printf '}\n'
  } > "$path"
}

grant() {
  local name="$1" filter="$2"
  jq "($filter)"' + {expires_at_unix_ms: 4102444800000, roots: {}}' "$logs/authority-inspect.stdout" \
    > "$work/$name-grant.json"
  printf 'import "%s"\n' "$name-grant.json" > "$work/$name-authority.ncl"
  config "$work/$name-config.ncl" "$work/$name-authority.ncl"
}

install -m 0444 "$LATTICE_CONSTRUCTION_EXAMPLES/local-shell-construction.ncl" "$out/source/"
install -m 0644 "$LATTICE_CONSTRUCTION_EXAMPLES/local-shell-construction.ncl" "$work/source/"
sed 's/"left" "stdin"/"missing" "stdin"/' "$out/source/local-shell-construction.ncl" > "$work/source/wrong-scope.ncl"
sed 's/g.generate 2 /g.generate 1 /' "$out/source/local-shell-construction.ncl" > "$work/source/generation-overflow.ncl"
config "$work/config.ncl"
cfg=(--config "$work/config.ncl")
cd "$work/source"

record import-construction pass "$lattice" "${cfg[@]}" import local-shell-construction.ncl --construction
record authoring-inspect pass "$lattice" "${cfg[@]}" authoring inspect "$workflow"
record export-author-source pass "$lattice" "${cfg[@]}" export "$workflow" "$out/source/$workflow.author.json"
record export-expanded pass "$lattice" "${cfg[@]}" export "$workflow" "$out/expanded/$workflow.ncl" --expanded
record negative-export-unlabeled-source fail "$lattice" "${cfg[@]}" export "$workflow" "$work/misleading.ncl"
expect negative-export-unlabeled-source-absent test ! -e "$work/misleading.ncl"
record authority-inspect pass "$lattice" "${cfg[@]}" authority inspect "$workflow"
for node in $(jq -r '.grants[].node_id' "$logs/authority-inspect.stdout"); do
  record "origin-node-$node" pass "$lattice" "${cfg[@]}" authoring origin "$workflow" "$node"
done
expect origin-template-instances test "$(grep -l '^template=worker instance=.* location=local-shell-construction.ncl:' "$logs"/origin-node-*.stdout | wc -l)" -eq 4

record negative-run-without-authority fail "$lattice" "${cfg[@]}" run "$workflow" --output json
grant foreign '.workflow_revision = ("blake3:" + ("0" * 64))'
record negative-run-with-foreign-authority fail "$lattice" --config "$work/foreign-config.ncl" run "$workflow" --output json
grant exact '.'
record run-with-exact-authority pass "$lattice" --config "$work/exact-config.ncl" run "$workflow" --output json
expect run-completed-every-node jq -e '.status == "succeeded" and .completed == 6 and .failures == 0' \
  "$logs/run-with-exact-authority.stdout"

record negative-import-wrong-scope-handle fail "$lattice" "${cfg[@]}" import wrong-scope.ncl --construction
record negative-import-generation-overflow fail "$lattice" "${cfg[@]}" import generation-overflow.ncl --construction
record authoring-inspect-after-negatives pass "$lattice" "${cfg[@]}" authoring inspect "$workflow"
expect rejected-imports-keep-admitted-revision cmp "$logs/authoring-inspect.stdout" "$logs/authoring-inspect-after-negatives.stdout"

for suite in test_construction_authoring test_construction_runtime_binding test_construction_qualification \
  test_source_authoring; do
  record "$suite" pass "$tests/$suite" --include-ignored --test-threads 1
done
record test_workflow_format-runtime-binding pass "$tests/test_workflow_format" \
  --ignored --exact flat_v3_file_import_selects_only_missing_runtime_bindings
record lattice-lib-source-origin-surface pass "$tests/lattice-lib" \
  --ignored --exact tui::authoring::tests::source_origins_target_only_matching_displayed_source_bytes

jq -R -s --argjson failed "${#failures[@]}" '
  split("\n") | map(select(length > 0) | split("\t")) | .[1:]
  | map({name: .[0], expected: .[1], exit_code: (.[2] | tonumber), command: .[3]})
  | {
      kind: "lattice-construction-runtime-host-qualification",
      unexpected_outcomes: $failed,
      steps: .,
      non_claims: [
        "No semantic equivalence between source, expanded graph and execution is claimed.",
        "No external service, producer, deployment or release qualification is claimed.",
        "Observations are bounded to the selected store artifacts on this host."
      ]
    }' "$commands" > "$out/summary.json"

if [ "${#failures[@]}" -ne 0 ]; then
  printf 'unexpected outcome: %s\n' "${failures[@]}" >&2
  exit 1
fi
echo "construction runtime qualification retained in $out"

set -eu
workspace=active
# Both compilation and the test binary are real processes in the owned group.
# Do not erase diagnostics or the failing source on a failed check.
printf '%s\n' started > "$workspace/evaluator-started"
if test "$(cat "$workspace/mode")" = cancel; then
    sleep 30
fi
status=0
rustc --edition 2024 --test "$workspace/build.rs" -o "$workspace/tests" > "$workspace/build.log" 2>&1 || status=$?
if test "$status" -eq 0; then
    "$workspace/tests" >> "$workspace/build.log" 2>&1 || status=$?
fi
if test "$status" -eq 0; then
    rustc --edition 2024 "$workspace/build.rs" -o "$workspace/program" >> "$workspace/build.log" 2>&1 || status=$?
fi
if test "$status" -eq 0; then
    "$workspace/program" >> "$workspace/build.log" 2>&1 || status=$?
fi
if test "$status" -eq 0; then
    printf '%s\n' pass > "$workspace/verdict.txt"
else
    printf 'fail exit=%s\n' "$status" > "$workspace/verdict.txt"
fi
cat "$workspace/verdict.txt"
exit "$status"

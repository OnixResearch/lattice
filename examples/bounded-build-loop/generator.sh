set -eu
workspace=$1
# This is a finite rule producer, not a model or a masked-repair adapter.
test "$(cat "$workspace/spec.txt")" = 'Return greeting() = "bounded build loop"; main prints it. The Rust unit test must pass.'
answer='incorrect first candidate'
if test "$(cat "$workspace/mode")" = pass || { test -f "$workspace/previous-verdict.txt" && test "$(cat "$workspace/mode")" != fail; }; then
    answer='bounded build loop'
fi
cat > "$workspace/build.rs" <<EOF
fn greeting() -> &'static str { "$answer" }
fn main() { println!("{}", greeting()); }
#[test]
fn required_greeting() { assert_eq!(greeting(), "bounded build loop"); }
EOF
cat "$workspace/build.rs"

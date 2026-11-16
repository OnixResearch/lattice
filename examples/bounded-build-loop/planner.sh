set -eu
workspace=$1
# The local rule planner has no tools, network, credentials or authority API.
printf '%s\n' 'Return greeting() = "bounded build loop"; main prints it. The Rust unit test must pass.' > "$workspace/spec.txt"
cat "$workspace/spec.txt"

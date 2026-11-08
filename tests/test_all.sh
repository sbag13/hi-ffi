#!/usr/bin/env bash
set -euo pipefail

tests_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

cargo build --manifest-path "$tests_dir/Cargo.toml"

(cd "$tests_dir/cpp" && ./test_cpp.sh --valgrind)
(cd "$tests_dir/swift" && ./test_swift.sh)
(cd "$tests_dir/python" && python3 "$tests_dir/python/main.py")
(cd "$tests_dir/java" && ./test_java.sh)
(cd "$tests_dir/c_sharp" && ./test_c_sharp.sh)

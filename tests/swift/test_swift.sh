#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
cd "$script_dir"

rm -rf CFfiModule
rm -rf FfiModule
rm -rf ModuleTest/.build

cp -r ../generated_code/swift/CFfiModule/ ./ && \
cp -r ../generated_code/swift/FfiModule/ ./ && \
cp ../target/debug/libtests.a . && \
cd ModuleTest && \
swift build -v -Xswiftc -L../

bin_path="$(swift build --show-bin-path)"
executable="$bin_path/ModuleTest"

if [ "${1:-}" = "--valgrind" ]; then
    valgrind --error-exitcode=1 --leak-check=full --show-leak-kinds=all "$executable"
else
    "$executable"
fi
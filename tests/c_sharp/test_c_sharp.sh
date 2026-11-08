#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
tests_dir="$(cd -- "$script_dir/.." && pwd)"

cd "$tests_dir"
cargo build

export LD_LIBRARY_PATH="$tests_dir/target/debug${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
dotnet run --project "$script_dir/CSharpTest.csproj"

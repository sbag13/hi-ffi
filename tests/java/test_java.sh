#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
tests_dir="$(cd -- "$script_dir/.." && pwd)"
classes_dir="$script_dir/classes"

cargo build --manifest-path "$tests_dir/Cargo.toml"

rm -rf "$classes_dir"
mkdir -p "$classes_dir"

javac --enable-preview --release 21 \
	-d "$classes_dir" \
	"$tests_dir/generated_code/java"/*.java \
	"$script_dir/Main.java"

java --enable-preview --enable-native-access=ALL-UNNAMED \
	-Djava.library.path="$tests_dir/target/debug" \
	-cp "$classes_dir" \
	Main

rm -rf CFfiModule
rm -rf FfiModule
cargo clean

cargo build && \
cp -r ../generated_code/swift/CFfiModule/ . && \
cp -r ../generated_code/swift/FfiModule/ . && \
cp ../target/debug/libtests.a . && \
cd ModuleTest && \
rm -rf ./build && \
swift build -v -Xswiftc -L../ && \
./.build/x86_64-unknown-linux-gnu/debug/ModuleTest
# valgrind --error-exitcode=2 --leak-check=full --show-leak-kinds=all ./.build/x86_64-unknown-linux-gnu/debug/ModuleTest
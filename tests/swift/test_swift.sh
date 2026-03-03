rm -rf CFfiModule
rm -rf FfiModule
rm -rf ModuleTest/.build

cp -r ../generated_code/swift/CFfiModule/ ./ && \
cp -r ../generated_code/swift/FfiModule/ ./ && \
cp ../target/debug/libtests.a . && \
cd ModuleTest && \
rm -rf ./build && \
swift build -v -Xswiftc -L../ && \

if [ "$1" = "--valgrind" ]; then
    valgrind --error-exitcode=1 --leak-check=full --show-leak-kinds=all ./.build/x86_64-unknown-linux-gnu/debug/ModuleTest
else
    ./.build/x86_64-unknown-linux-gnu/debug/ModuleTest
fi
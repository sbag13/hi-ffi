mkdir -p lib
cp ../target/debug/libtests.a lib/
g++ ../generated_code/cpp/*.cpp main.cpp assertions.cpp \
    -I ../generated_code/cpp/ \
    -L ./lib \
    -l tests \
    -o test && \
./test
# valgrind --error-exitcode=1 --leak-check=full --show-leak-kinds=all ./test

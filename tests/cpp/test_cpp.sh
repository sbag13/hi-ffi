cargo build && \
g++ ../generated_code/cpp/* main.cpp \
    -I ../generated_code/cpp/ \
    -L ../target/debug/ \
    -l tests \
    -o test && \
./test && \
valgrind --error-exitcode=2 --leak-check=full --show-leak-kinds=all ./test
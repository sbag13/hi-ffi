cd cpp && ./test_cpp.sh --valgrind || echo "C++ tests failed"; cd -
cd swift && ./test_swift.sh || echo "Swift tests failed"; cd -
cd python && python3 main.py || echo "Python tests failed"; cd -
cd java && ./test_java.sh || echo "Java tests failed"; cd -

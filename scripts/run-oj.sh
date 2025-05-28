echo "start to run glibc"
cd /glibc
./basic_testcode.sh
./busybox_testcode.sh
./lua_testcode.sh
./libctest_testcode.sh
./iozone_testcode.sh
./libcbench_testcode.sh
./lmbench_testcode.sh
cd ..
echo "finish to run glibc"

echo "start to run musl"
cd /musl
./basic_testcode.sh
./busybox_testcode.sh
./lua_testcode.sh
./libctest_testcode.sh
./iozone_testcode.sh
./libcbench_testcode.sh
./lmbench_testcode.sh
cd ..

exit
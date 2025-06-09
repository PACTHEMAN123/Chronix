echo "start to test loongarch"

echo "start to set up libs"
cp /loongarch/glibc/lib/ld-linux-loongarch-lp64d.so.1 /lib64/ld-linux-loongarch-lp64d.so.1
cp /loongarch/glibc/lib/libc.so.6 /lib64/libc.so.6
cp /loongarch/glibc/lib/libm.so.6 /lib64/libm.so.6
cp /loongarch/glibc/lib/libc.so.6 /usr/lib64/libc.so.6
cp /loongarch/glibc/lib/libm.so.6 /usr/lib64/libm.so.6
cp /loongarch/musl/lib/libc.so /lib/ld-musl-loongarch64-lp64d.so.1
cp /loongarch/musl/lib/libc.so /lib64/ld-musl-loongarch-lp64d.so.1
echo "finish set up"

echo "start to run glibc"
cd /loongarch/glibc
./basic_testcode.sh
./busybox_testcode.sh
./lua_testcode.sh
./libctest_testcode.sh
./netperf_testcode.sh
# ./iozone_testcode.sh
./libcbench_testcode.sh
# ./lmbench_testcode.sh
./cyclictest_testcode.sh
cd ..
echo "finish to run glibc"

echo "start to run musl"
cd /loongarch/musl
./basic_testcode.sh
./busybox_testcode.sh
./lua_testcode.sh
./libctest_testcode.sh
./netperf_testcode.sh
# ./iozone_testcode.sh
./libcbench_testcode.sh
# ./lmbench_testcode.sh
./cyclictest_testcode.sh
cd ..

exit
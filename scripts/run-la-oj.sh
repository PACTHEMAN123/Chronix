echo "start to test loongarch"

echo "start to set up libs"
ln -s /loongarch/glibc/lib/ld-linux-loongarch-lp64d.so.1 /lib64/ld-linux-loongarch-lp64d.so.1
ln -s /loongarch/glibc/lib/libc.so.6 /lib64/libc.so.6
ln -s /loongarch/glibc/lib/libm.so.6 /lib64/libm.so.6
ln -s /loongarch/glibc/lib/libc.so.6 /usr/lib64/libc.so.6
ln -s /loongarch/glibc/lib/libm.so.6 /usr/lib64/libm.so.6
ln -s /loongarch/musl/lib/libc.so /lib/ld-musl-loongarch64-lp64d.so.1
ln -s /loongarch/musl/lib/libc.so /lib64/ld-musl-loongarch-lp64d.so.1
echo "finish set up"

echo "start to run glibc"
cd /loongarch/glibc
./basic_testcode.sh
./busybox_testcode.sh
./lua_testcode.sh
./libctest_testcode.sh
./netperf_testcode.sh
./iperf_testcode.sh
./libcbench_testcode.sh
cd ..
echo "finish to run glibc"

echo "start to run musl"
cd /loongarch/musl
./basic_testcode.sh
./busybox_testcode.sh
./lua_testcode.sh
./libctest_testcode.sh
./netperf_testcode.sh
./iperf_testcode.sh
./libcbench_testcode.sh
cd ..
echo "finish to run musl"

echo "start to run iz"
cd /loongarch/glibc
./iozone_testcode.sh
cd ..
cd /loongarch/musl
./iozone_testcode.sh
cd ..

exit
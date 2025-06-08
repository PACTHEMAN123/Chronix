echo "start to test riscv"

echo "start to set up libs"
cp /riscv/glibc/lib/ld-linux-riscv64-lp64d.so.1 /lib/ld-linux-riscv64-lp64d.so.1
cp /riscv/glibc/lib/ld-linux-riscv64-lp64d.so.1 /lib/ld-linux-riscv64-lp64.so.1
cp /riscv/glibc/lib/libc.so /lib/libc.so.6
cp /riscv/glibc/lib/libm.so /lib/libm.so.6
cp /riscv/musl/lib/libc.so /lib/ld-musl-riscv64-sf.so.1
echo "finish set up"

echo "start to run glibc"
cd /riscv/glibc
./basic_testcode.sh
./busybox_testcode.sh
./lua_testcode.sh
./libctest_testcode.sh
./netperf_testcode.sh
./libcbench_testcode.sh
./cyclictest_testcode.sh
cd ..
echo "finish to run glibc"

echo "start to run musl"
cd /riscv/musl
./basic_testcode.sh
./busybox_testcode.sh
./lua_testcode.sh
./libctest_testcode.sh
./netperf_testcode.sh
./libcbench_testcode.sh
./cyclictest_testcode.sh
cd ..
echo "finish to run musl"

echo "start to run iz"
cd /riscv/glibc
./iozone_testcode.sh
cd ..
cd /riscv/musl
./iozone_testcode.sh
cd ..

exit
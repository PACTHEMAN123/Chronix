echo "start to test riscv"

echo "start to set up libs"
ln -s /riscv/glibc/lib/ld-linux-riscv64-lp64d.so.1 /lib/ld-linux-riscv64-lp64d.so.1
ln -s /riscv/glibc/lib/ld-linux-riscv64-lp64d.so.1 /lib/ld-linux-riscv64-lp64.so.1
ln -s /riscv/glibc/lib/libc.so /lib/libc.so.6
ln -s /riscv/glibc/lib/libm.so /lib/libm.so.6
ln -s /riscv/musl/lib/libc.so /lib/ld-musl-riscv64-sf.so.1
ln -s /riscv/musl/lib/libc.so /lib/ld-musl-riscv64.so.1
echo "finish set up"

echo "start to test glibc"
cd /
cd sdcard/glibc
./splice_testcode.sh
./copy-file-range_testcode.sh
./interrupts_testcode.sh
echo "finish running glibc"


echo "start to test musl"
cd /
cd sdcard/musl
./splice_testcode.sh
./copy-file-range_testcode.sh
./interrupts_testcode.sh
echo "finish running musl"

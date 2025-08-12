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


echo "start to test ltp in musl"
cd /
cd sdcard/musl/ltp/testcases/bin

echo "#### OS COMP TEST GROUP START ltp-musl ####"

file_list="
accept01
chmod01
chown01 chown02 chown05
close01 close02
creat03 creat05
dup01 dup02 dup03 dup04 dup06 dup07 dup202 dup203 dup204 dup206 dup207 dup3_01 dup3_02
faccessat01 faccessat02 faccessat201
fallocate03
fanotify04
"
set -- $file_list

for file in $@; do
  # 跳过目录，仅处理文件
  if [ -f "$file" ]; then
    # 输出文件名
    echo "RUN LTP CASE $(basename "$file")"

    "./$file"
    ret=$?

    # 输出文件名和返回值
    echo "FAIL LTP CASE $(basename "$file") : $ret"
  fi
done


echo "#### OS COMP TEST GROUP END ltp-musl ####"


echo "start to test ltp in musl"
cd /
cd sdcard/musl/ltp/testcases/bin

echo "#### OS COMP TEST GROUP START ltp-musl ####"

file_list="
accept01 accept4_01
access01 access02 access03
add_key01 add_key04
adjtimex01
alarm02 alarm03 alarm05 alarm06 alarm07
bind01 bind04 bpf_map01
capget01
chdir04
chmod01 chmod03
chown01 chown02 chown05
clock_getres01
clone01 clone03 clone06
close01 close02
confstr01
creat01 creat03 creat05 creat08
dio_sparse
dup01 dup02 dup03 dup04 dup06 dup07 dup202 dup203 dup204 dup206 dup207 dup3_01 dup3_02
faccessat01 faccessat02 faccessat201
fallocate03
fanotify08
fchdir01 fchdir02
fchmod01 fchmod03 fchmod04 fchmod06
fchmodat01 fchmodat02
fchown01 fchown02 fchown03 fchown04 fchown05
fchownat01 fchownat02
fcntl02 fcntl02_64 fcntl03 fcntl03_64 fcntl04 fcntl04_64 fcntl05 fcntl05_64 fcntl08 fcntl08_64
fcntl12 fcntl12_64 fcntl29 fcntl29_64 fcntl34 fcntl34_64 fcntl36 fcntl36_64
fork01 fork03 fork04 fork07 fork08 fpathconf01
fstat02 fstat02_64 fstat03 fstat03_64
fstatfs02 fstatfs02_64
ftruncate01 ftruncate01_64
getdomainname01
geteuid01 geteuid02
mkdir05
mkdirat01
tgkill01 tgkill03
time01
times01
tkill01
uname01
unlink05
unlink07
unlinkat01
utime06 utime07
utimes01
utsname01 utsname04
wait01 wait02 wait401
waitpid01 waitpid03
write01 write02 write06
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
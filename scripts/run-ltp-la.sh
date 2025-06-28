

echo "start to test ltp in musl"
cd /
cd sdcard/musl/ltp/testcases/bin



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
chown01 chown02 chown03 chown05
clock_getres01 
clock_gettime02
clock_nanosleep01 clock_nanosleep04
clone01 clone03 clone06
close01 close02
confstr01
creat01 creat03 creat05 creat08

dup01 dup02 dup03 dup04 dup06 dup07 dup202 dup203 dup204 dup206 dup207 dup3_01 dup3_02

epoll_create01 epoll_create1_01
epoll_ctl03
execl01 
execle01
execlp01
execve01
execve03
execvp01
exit02
exit_group01

faccessat01 faccessat02 faccessat201 faccessat202
fallocate03
fanotify08
fchdir01 fchdir02
fchmod01 fchmod03 fchmod04 fchmod06
fchmodat01 fchmodat02
fchown01 fchown02 fchown03 fchown04 fchown05
fchownat01 fchownat02
fcntl02 fcntl02_64 fcntl03 fcntl03_64 fcntl04 fcntl04_64 fcntl05 fcntl05_64 fcntl08 fcntl08_64
fcntl12 fcntl12_64 fcntl29 fcntl29_64 fcntl34 fcntl34_64 fcntl36 fcntl36_64
flistxattr03
flock01 flock03 flock04 flock06
fork01 fork03 fork04 fork07 fork08 fpathconf01
fstat02 fstat02_64 fstat03 fstat03_64
fstatfs02 fstatfs02_64
ftruncate01 ftruncate01_64
futex_wait01

geteuid01 geteuid02
gethostname01
getitimer01 getitimer02
getpagesize01
getpeername01
getpgid01
getpgrp01
getpid02
getppid02
getrandom01 getrandom02 getrandom03 getrandom04 getrandom05
getrlimit01 getrlimit02
getrusage01 getrusage02
gettid02
gettimeofday01 ettimeofday02
getuid01 getuid03
getdomainname01

in6_01
inotify_init1_01 inotify_init1_02
ioprio_set02
io_uring01

kcmp01 kcmp03
keyctl04

lftest
link02 link04 link05
listxattr03 llistxattr03
llseek01 llseek02 llseek03
lseek01

madvise01 madvise05 madvise10
memset01
mesgq_nstest
mincore02
mincore03
mkdir05
mkdirat01
mlock01 mlock04
mmap02 mmap05 mmap06 mmap09 mmap19
mprotect05
mtest01
munlock01
name_to_handle_at01

open01 open02 open08 open10 open11
openat01

pipe01 pipe06 pipe10 pipe14 pipe2_01
poll01
posix_fadvise01 posix_fadvise01_64
prctl01
pselect03 pselect03_64
pwrite02 pwrite02_64 pwrite03 pwrite03_64 pwrite04 pwrite04_64

read01 read02 read04
readdir01
readlinkat01 readlinkat02
readv01 readv02

rmdir03
rtc02
rt_sigsuspend01

sbrk01 sbrk02
sched_getaffinity01
sendfile02 sendfile02_64 sendfile03 sendfile03_64 sendfile05 sendfile05_64
setrlimit02 setrlimit04 setrlimit05
setuid01
setxattr02
signal02 signal03 signal04 signal05
sigpending02
sigwait01
socket01
socketpair01
socketpair02
splice02
stat01 stat01_64 stat02 stat02_64
statx01 statx02 statx03
symlink04
syscall01

tgkill01 tgkill03
time01
times01
tkill01

uname01
unlink05 unlink07
unlinkat01
utime06 utime07
utimes01
utsname01 utsname04

wait01 wait02 wait401
waitpid01 waitpid03
write01 write02 write06
"
set -- $file_list

echo "start to test ltp in musl"
cd /
cd sdcard/musl/ltp/testcases/bin

echo "#### OS COMP TEST GROUP START ltp-musl ####"

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

echo "start to test ltp in glibc"
cd /
cd sdcard/glibc/ltp/testcases/bin


echo "#### OS COMP TEST GROUP START ltp-glibc ####"

for file in $@; do
  if [ -f "$file" ]; then
    echo "RUN LTP CASE $(basename "$file")"

    "./$file"
    ret=$?

    echo "FAIL LTP CASE $(basename "$file") : $ret"
  fi
done


echo "#### OS COMP TEST GROUP END ltp-glibc ####"
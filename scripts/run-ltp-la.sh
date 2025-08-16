

echo "start to test ltp in musl"
cd /
cd sdcard/musl/ltp/testcases/bin



file_list="
abort01
accept01 accept03 accept4_01
access01 access02 access03 access04
add_key01 add_key04
adjtimex01 adjtimex02 adjtimex03
af_alg01 af_alg02 af_alg03
alarm02 alarm03 alarm05 alarm06 alarm07

bind01 bind03 bind04 bpf_map01
brk01 brk02

capget01
chdir01 chdir04
chmod01 chmod03 chmod06
chown01 chown02 chown03 chown04 chown05
clock_adjtime01 clock_adjtime02
clock_getres01 
clock_gettime02 clock_gettime04
clock_nanosleep01 clock_nanosleep04
clock_settime01 clock_settime02
clone01 clone03 clone06 clone08 clone302
close01 close02
confstr01
copy_file_range01 copy_file_range03
creat01 creat03 creat05 creat06 creat08

dio_sparse
dup01 dup02 dup03 dup04 dup06 dup07 dup202 dup203 dup204 dup206 dup207 dup3_01 dup3_02

epoll_create01 epoll_create1_01 epoll_create1_02
epoll_ctl02 epoll_ctl03
epoll_wait01 epoll_wait03 epoll_wait04 epoll_wait06
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
fanotify01 fanotify08 fanotify14
fchdir01 fchdir02
fchmod01 fchmod03 fchmod04 fchmod06
fchmodat01 fchmodat02
fchown01 fchown02 fchown03 fchown04 fchown05
fchownat01 fchownat02
fcntl02 fcntl02_64 fcntl03 fcntl03_64 fcntl04 fcntl04_64 fcntl05 fcntl05_64 fcntl08 fcntl08_64
fcntl12 fcntl12_64 fcntl29 fcntl29_64 fcntl34 fcntl34_64 fcntl36 fcntl36_64
fgetxattr01 
flistxattr03
flock01 flock03 flock04 flock06
fork01 fork03 fork04 fork07 fork08 fork10 fpathconf01
fstat02 fstat02_64 fstat03 fstat03_64
fstatfs01 fstatfs01_64 fstatfs02 fstatfs02_64
fsync01 fsync02
ftruncate01 ftruncate01_64 ftruncate03 ftruncate03_64
futex_wait01 futex_wait04

getcwd01 getcwd02
getdents02
getdomainname01
getegid02
geteuid01 geteuid02
getgid01 getgid03
gethostname01
getitimer01 getitimer02
getpagesize01
getpeername01
getpgid01 getpgid02
getpgrp01
getpid01 getpid02
getppid01 getppid02
getpriority01 getpriority02
getrandom01 getrandom02 getrandom03 getrandom04 getrandom05
getrlimit01 getrlimit02
getrusage01 getrusage02
getsockname01
getsockopt01
gettid02
gettimeofday01 gettimeofday02
getuid01 getuid03

in6_01
inotify_init1_01 inotify_init1_02
ioprio_set02
io_uring01

kcmp01 kcmp03
keyctl04
kill06 kill11

lftest
link02 link04 link05
listxattr03 llistxattr03
llseek01 llseek02 llseek03
lseek01 lseek07
lstat02 lstat02_64

madvise01 madvise05 madvise10
memcmp01
memcpy01
memset01
mesgq_nstest
mincore02 mincore03
mkdir03 mkdir05 mkdir09
mknod01 mknod02
mlock01 mlock04
mmap02 mmap05 mmap06 mmap09 mmap19
mprotect05
mq_notify01 mq_notify02 mq_notify03
mq_open01 mq_timedsend01 mq_unlink01
msgctl01 msgctl02 msgctl03 msgctl12
msgrcv07
mtest01
munlock01

name_to_handle_at01
nanosleep04
nice01 nice02 nice03

open01 open02 open08 open10 open11
openat01
open_by_handle_at01

pathconf01 pathconf02
personality02
pidns32
pipe01 pipe03 pipe06 pipe07 pipe10 pipe11 pipe12 pipe14 pipe2_01
poll01
posix_fadvise01 posix_fadvise01_64 
posix_fadvise02 posix_fadvise02_64 
posix_fadvise03 posix_fadvise03_64
posix_fadvise04 posix_fadvise04_64
ppoll01
prctl01 prctl03 prctl04 prctl05 prctl08
pread01 pread01_64
pread02 pread02_64
preadv01 preadv01_64
preadv02 preadv02_64
preadv201 preadv201_64
preadv202 preadv202_64
pselect01 pselect01_64
pselect02 pselect02_64
pselect03 pselect03_64
pwrite01 pwrite01_64
pwrite02 pwrite02_64
pwrite03 pwrite03_64
pwrite04 pwrite04_64
pwritev01 pwritev01_64
pwritev02 pwritev02_64
pwritev201 pwritev201_64
pwritev202 pwritev202_64

read01 read02 read04
readahead01
readdir01
readlink01 readlink03
readlinkat01 readlinkat02
readv01 readv02
rename08 rename10
rmdir01 rmdir02
rtc02
rt_sigsuspend01

sbrk01 sbrk02
sched_getaffinity01
sched_getscheduler01
select03
sendfile02 sendfile02_64
sendfile03 sendfile03_64 
sendfile04 sendfile04_64 
sendfile05 sendfile05_64
sendfile06 sendfile06_64
sendfile08 sendfile08_64
sendmmsg02
sendto02
setdomainname01 setdomainname02
setegid01
setgid01 setgid03
setgroups01 setgroups02
sethostname01 sethostname02
setitimer02
setpgrp02
setpriority01 setpriority02
setregid01 setregid03 setregid04
setresgid02
setresuid01 setresuid02 setresuid04 setresuid05
setreuid01 setreuid02 setreuid03 setreuid04 setreuid05 setreuid07
setrlimit02 setrlimit04 setrlimit05
setsockopt01 setsockopt02 setsockopt03 setsockopt04
settimeofday01 settimeofday02
setuid01
setxattr01 setxattr02
shmctl07
signal02 signal03 signal04 signal05
sigpending02
sigwait01
socket01 socket02
socketpair01
socketpair02
splice01 splice02 splice03 splice04 splice07
stat01 stat01_64
stat02 stat02_64
stat03 stat03_64
statfs02 statfs02_64
statvfs02
statx01 statx02 statx03
stime01
symlink02 symlink04
syscall01

tgkill01 tgkill03
time01
timer_delete01 timer_delete02
timerfd02
timerfd_create01
timerfd_gettime01 
timerfd_settime01 timerfd_settime02
timer_getoverrun01
timer_gettime01 timer_settime02
times01
tkill01 tkill02

umount01
uname01 uname02 uname04
unlink05 unlink07
unlinkat01
utime01 utime02 utime04 utime05 utime06 utime07
utimensat01
utimes01
utsname01 utsname02 utsname03 utsname04

vmsplice01 vmsplice02 vmsplice03

wait01 wait02 wait401 wait402
waitpid01 waitpid03 waitpid04
write01 write02 write03 write05 write06
writev01 writev07
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
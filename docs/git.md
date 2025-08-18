# setup

```bash
./attach/env-rv.sh
```


# operate

```bash
mkdir tt && cd tt && git init && git config user.email "oscomp@gmail.com" && git config user.name "oscomp" && echo "hello" > README.md

cat >README.md (hang?)



git add README.md

git ls-files --stage

git commit -m "add readme"

git log
```


```bash
# 手动将文件加入索引
file_hash=$(git hash-object -w README.md) && git update-index --add --cacheinfo 100644,$file_hash,"README.md"
```


[ INFO] task 3 trying to open .git/index, oflags: (empty), atflags: AT_STATX_SYNC_AS_STAT, dirfd -100
[ WARN] cannot open .git/index, not exist

[ INFO] fstatat dirfd -100, path /tt/.git/index.lock, at_flags AT_STATX_SYNC_AS_STAT, oflags (empty)
[ INFO] [sys_fstatat]: /tt/.git/index.lock size 104
[ INFO] task 3, syscall: SYSCALL_RENAMEAT2, args: [ffffffffffffff9c, 307610, ffffffffffffff9c, 307a40, 0, 307610]
[ INFO]  rename /tt/.git/index.lock -> /tt/.git/index, using flags (empty)
[ INFO] old inode size 104
[ INFO] [Ext4] rename /tt/.git/index.lock -> /tt/.git/index
[ INFO] old dentry /tt/.git/index.lock, old inode size 104; new dentry /tt/.git/index, new inode size 104
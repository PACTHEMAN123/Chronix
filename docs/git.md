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
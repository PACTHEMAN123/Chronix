

./attach/env-rv.sh && cd /attach/riscv

./sqlite3 tasks.db

CREATE TABLE tasks (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    status TEXT NOT NULL,
    priority INTEGER
);
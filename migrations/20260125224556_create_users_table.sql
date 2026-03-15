-- Users table
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    passhash TEXT NOT NULL
);

-- example (TODO: change passhash)
INSERT OR IGNORE INTO users (username, passhash) VALUES ('admin', 'admin');


-- Watchers table

-- Desativa chaves estrangeiras temporariamente para a troca de tabelas
PRAGMA foreign_keys=OFF;

CREATE TABLE new_watchers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    url TEXT NOT NULL,
    created_by INTEGER NOT NULL DEFAULT 1 REFERENCES users(id) ON DELETE CASCADE
);

-- Copia apenas os dados existentes (id, url) e define o admin (id 1) como criador
INSERT INTO new_watchers (id, url, created_by)
    SELECT id, url, 1 FROM watchers;

DROP TABLE watchers;

ALTER TABLE new_watchers RENAME TO watchers;

-- Reativa chaves estrangeiras e confirma as alterações
PRAGMA foreign_keys=ON;

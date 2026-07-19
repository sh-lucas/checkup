CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    passhash TEXT NOT NULL
);

-- example (TODO: change passhash)
INSERT OR IGNORE INTO users (username, passhash) VALUES ('admin', 'admin');

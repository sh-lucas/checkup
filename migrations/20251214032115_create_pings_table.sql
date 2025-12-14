CREATE TABLE pings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    watcher_id INTEGER NOT NULL,
    timestamp DATETIME NOT NULL,
    status_code INTEGER NOT NULL,
    FOREIGN KEY (watcher_id) REFERENCES watchers (id)
);
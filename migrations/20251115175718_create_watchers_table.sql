CREATE TABLE IF NOT EXISTS watchers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    url TEXT NOT NULL,
    created_by INTEGER NOT NULL DEFAULT 1
);

INSERT INTO watchers (url) VALUES ('https://google.com');

-- Creates the watchers table =)
CREATE TABLE IF NOT EXISTS watchers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    url TEXT NOT NULL
    -- interval INTEGER NOT NULL
);

INSERT INTO watchers (url) VALUES ('https://google.com');
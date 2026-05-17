-- Seed data — run after migrations (server first boot creates the tables)
PRAGMA journal_mode=WAL;
PRAGMA cache_size=-5000;

INSERT OR IGNORE INTO users (username, passhash) VALUES
  ('user1','x'),('user2','x'),('user3','x'),('user4','x'),('user5','x'),
  ('user6','x'),('user7','x'),('user8','x'),('user9','x'),('user10','x');

INSERT OR IGNORE INTO watchers (url, created_by)
SELECT 'https://example-bench.com/w' || n, ((n-1) % 10) + 1
FROM (WITH RECURSIVE r(n) AS (SELECT 1 UNION ALL SELECT n+1 FROM r WHERE n<50) SELECT n FROM r);

INSERT INTO pings (watcher_id, timestamp, status_code, status)
SELECT
  ((abs(random()) % 50) + 1),
  datetime('now', '-' || (abs(random()) % 43200) || ' minutes'),
  CASE WHEN abs(random()) % 10 = 0 THEN 503 ELSE 200 END,
  CASE WHEN abs(random()) % 10 = 0 THEN 'offline' ELSE 'online' END
FROM (WITH RECURSIVE r(n) AS (SELECT 1 UNION ALL SELECT n+1 FROM r WHERE n<20000) SELECT n FROM r);

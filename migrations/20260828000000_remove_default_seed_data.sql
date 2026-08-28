-- Do not provision default credentials or a default watcher.
-- The predicates keep this safe for databases where these seed records were
-- already removed or replaced.
DELETE FROM users
WHERE email = 'admin'
  AND password_hash = 'admin';

DELETE FROM watchers
WHERE url = 'https://google.com'
  AND created_by = 1;

-- Add index on timestamp for range queries and ordering
CREATE INDEX IF NOT EXISTS idx_pings_timestamp ON pings (timestamp);

-- Add composite index on watcher_id and status for filtering down status checks
CREATE INDEX IF NOT EXISTS idx_pings_watcher_status ON pings (watcher_id, status);

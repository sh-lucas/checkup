-- Create system uptime events table
CREATE TABLE IF NOT EXISTS system_uptime_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    reference TEXT NOT NULL,
    event INTEGER NOT NULL CHECK (event IN (0, 1)), -- 0 = online_until, 1 = offline_until
    timestamp TEXT NOT NULL
);

-- Index for querying system events by reference and timestamp
CREATE INDEX IF NOT EXISTS idx_system_uptime_events_ref_time 
ON system_uptime_events (reference, timestamp);

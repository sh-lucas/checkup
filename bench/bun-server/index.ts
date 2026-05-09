import { Database } from "bun:sqlite";

const dbPath = (process.env.DATABASE_URL ?? "bench/bench.db").replace("sqlite:///", "");

const db = new Database(dbPath, { create: true });
db.exec("PRAGMA journal_mode=WAL");
db.exec("PRAGMA cache_size=-5000");
db.exec("PRAGMA busy_timeout=5000");
db.exec(`
  CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    passhash TEXT NOT NULL
  );
  CREATE TABLE IF NOT EXISTS watchers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    url TEXT NOT NULL,
    created_by INTEGER NOT NULL DEFAULT 1 REFERENCES users(id) ON DELETE CASCADE
  );
  CREATE TABLE IF NOT EXISTS pings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    watcher_id INTEGER NOT NULL,
    timestamp DATETIME NOT NULL,
    status_code INTEGER NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('online','offline')),
    FOREIGN KEY (watcher_id) REFERENCES watchers(id)
  );
`);

const heavyQ = db.query(`
  SELECT u.username, w.url, COUNT(p.id) AS ping_count,
         MAX(p.timestamp) AS last_ping, MAX(p.status) AS last_status
  FROM users u
  JOIN watchers w ON w.created_by = u.id
  LEFT JOIN pings p ON p.watcher_id = w.id
  GROUP BY u.id, w.id ORDER BY ping_count DESC LIMIT 50
`);
const countUsersQ    = db.query("SELECT COUNT(*) AS n FROM users");
const countWatchersQ = db.query("SELECT COUNT(*) AS n FROM watchers");
const countPingsQ    = db.query("SELECT COUNT(*) AS n FROM pings");
const countOnlineQ   = db.query("SELECT COUNT(*) AS n FROM pings WHERE status='online'");
const countOfflineQ  = db.query("SELECT COUNT(*) AS n FROM pings WHERE status='offline'");
const insertPingQ    = db.query(
  "INSERT INTO pings (watcher_id, timestamp, status_code, status) VALUES (1, datetime('now'), 200, 'online') RETURNING id"
);

const PORT = parseInt(process.env.PORT ?? "3000");
Bun.serve({
  port: PORT,
  fetch(req) {
    const { pathname } = new URL(req.url);
    if (pathname === "/")              return Response.json({ message: "server online" });
    if (pathname === "/bench/heavy")   return Response.json({ rows: heavyQ.all() });
    if (pathname === "/bench/light")   return Response.json({
      user_count:    (countUsersQ.get()    as any).n,
      watcher_count: (countWatchersQ.get() as any).n,
      ping_count:    (countPingsQ.get()    as any).n,
      online_count:  (countOnlineQ.get()   as any).n,
      offline_count: (countOfflineQ.get()  as any).n,
    });
    if (pathname === "/bench/write" && req.method === "POST")
      return Response.json({ inserted_id: (insertPingQ.get() as any).id });
    return new Response("Not Found", { status: 404 });
  },
});
console.log(`Bun listening on http://0.0.0.0:${PORT}`);

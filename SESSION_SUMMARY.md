# Session Summary — Turso Integration

## Session 2 — Stub cleanup & TCL fix (2026-04-28)

**Problema:** PR bloqueado pelo CI com `undefined symbol: sqlite3_search_count` no TCL conformance test.

**Root cause:** A variável global `sqlite3_search_count` (esperada como `extern int` pelo binding TCL) nunca foi exportada no `sqlite3/src/lib.rs`.

**Fix:** Adicionado `#[no_mangle] pub static mut sqlite3_search_count: ffi::c_int = 0;`.

**Stubs removidos:** Todas as funções com `stub!()/todo!()` foram substituídas:
- `sqlite3_sleep` → `thread::sleep` real
- `sqlite3_stricmp` → `libc::strcasecmp`
- `sqlite3_complete` → checa se último char não-espaço é `;`
- `sqlite3_stmt_busy` → checa se há row pendente
- callbacks opcionais (`trace_v2`, `progress_handler`, `set_authorizer`, etc.) → `SQLITE_OK`
- features não implementadas (`backup_*`, `blob_*`, `serialize`) → `SQLITE_ERROR`/`null`

**Estado do fork:** `git@github.com:sh-lucas/turso-sqlx-patch.git` branch `main`, commit `67ef556f5`. Base atualizada para upstream mais recente (`8121b0875`).

---

## Context

The goal was to integrate [Turso Database](https://github.com/tursodatabase/turso) (formerly Limbo — a SQLite rewrite in Rust) as the SQLite backend for this project, replacing the bundled SQLite that sqlx ships with.

The approach: compile `turso_sqlite3` as a cdylib, symlink it as `libsqlite3.so`, and point `libsqlite3-sys` at it via `SQLITE3_LIB_DIR` + `pkg-config`.

## Project Overview

**checkup** — A healthcheck API written in Rust.
- **Web framework:** [Poem](https://github.com/poem-web/poem)
- **Database:** SQLite via [sqlx](https://github.com/launchbadge/sqlx)
- **Auth:** JWT (jsonwebtoken)
- **Runtime:** Tokio

### Features
| Endpoint | Method | Description |
|----------|--------|-------------|
| `/` | GET | Health check — returns `{"message":"server online"}` |
| `/watchers/create` | POST | Register a URL to monitor |
| `/watchers/list` | GET | List watchers (requires Bearer auth) |
| `/users/create` | POST | Create user, returns JWT tokens |
| `/pings` | GET | Get offline pings for a watcher |

### Background Worker
A Tokio task runs every 10s, streams all watchers from the DB, pings each URL, and logs status changes (`offline`) when status >= 400.

## Git State

| Branch | Description |
|--------|-------------|
| `master` | Main branch, SQLite bundled |
| `refac/turso-database-migration` | Turso integration branch (this session) |

The Turso repo lives in `/turso/` (gitignored) and is a separate clone with:
- `origin` → `git@github.com:sh-lucas/turso-sqlx-patch.git` (your fork)
- `upstream` → `https://github.com/tursodatabase/turso.git` (official)

## Bugs Found & Fixed

### BO #1: `PRAGMA foreign_keys` crashes Turso

**Symptom:** `SqliteError { code: 21, message: "bad parameter or other API misuse" }`

**Root cause:** sqlx sends `PRAGMA foreign_keys = ON;` on every connection. Turso (Limbo) doesn't implement this pragma and returns `SQLITE_MISUSE`. The migration SQL also had `PRAGMA foreign_keys=OFF/ON;` statements.

**Fix (3 layers):**
1. **`turso/sqlite3/src/lib.rs`** — Intercept `PRAGMA foreign_keys` in `sqlite3_prepare_v2`, replace with `SELECT 1`. Also fixed `_len` parameter handling (was ignored, causing tail pointer arithmetic to break).
2. **`migrations/20260125224556_create_users_table.sql`** — Removed the `PRAGMA foreign_keys=OFF;` and `PRAGMA foreign_keys=ON;` statements.
3. **`src/database.rs`** — Use `SqliteConnectOptions` without calling `.foreign_keys()` (the default `None` doesn't send any PRAGMA on connect).

### BO #2: `pings_repository` INSERT with swapped columns

**Symptom:** The `status` string ("offline") was being inserted into the `status_code` INTEGER column, and the `status` TEXT column was never populated.

**Root cause:** Wrong parameter order in the SQL query.

**Fix:**
- Added `status_code: u16` parameter to `log_status_change()`
- Fixed the INSERT: `INSERT INTO pings (watcher_id, status_code, status, timestamp)`
- Changed `fetch_one` → `fetch_optional` (crashes on first ping when no previous log exists)
- Updated the caller in `worker.rs` to pass `status_code`

### BO #3: `sqlite3_value_blob` returns NULL for TEXT values

**Symptom:** `assertion failed: !ptr.is_null()` in `sqlx-sqlite-0.8.6/src/value.rs:168`

**Root cause:** Turso's `sqlite3_value_blob` only handled `Blob` values, ignoring `Text`. sqlx uses `sqlite3_value_blob` to read both BLOB and TEXT data.

**Fix:** In `turso/sqlite3/src/lib.rs`, handle `ValueType::Text` explicitly in `sqlite3_value_blob`, returning the raw text bytes (matching SQLite's actual behavior).

### BO #4: `sqlite3_column_type` panics without a row

**Symptom:** `Function should only be called after SQLITE_ROW`

**Root cause:** Turso's `sqlite3_column_type` called `.row().expect(...)` which panics if called before `sqlite3_step` returns `SQLITE_ROW`. Per SQLite docs, it should return `SQLITE_NULL` in this case.

**Fix:** Changed to `let Some(row) = stmt.stmt.row() else { return SQLITE_NULL; }`.

## Files Modified

### checkup repo (`refac/turso-database-migration`)

| File | Change |
|------|--------|
| `src/database.rs` | Use `SqliteConnectOptions` without foreign_keys setting |
| `src/features/pings/pings_repository.rs` | Fix INSERT columns, `fetch_optional`, add `status_code` param |
| `src/features/watchers/worker.rs` | Pass `status_code` to `log_status_change()` |
| `migrations/20260125224556_create_users_table.sql` | Remove `PRAGMA foreign_keys` statements |

### Turso fork (`turso-sqlx-patch`)

| File | Change |
|------|--------|
| `sqlite3/src/lib.rs` | 17 hunks: PRAGMA intercept, `_len` handling, `column_type` fix, `value_blob` fix, plus pre-existing setup differences |

The Turso fork is based on `upstream/main` (commit `0f6fdd7`) with 1 additional commit containing all patches.

## Patch File

A standalone patch file was saved to `~/turso-sqlx-patch.patch` (522 lines, 17 hunks). It applies cleanly on a fresh `--depth 1` clone of the official Turso repo:

```bash
git clone --depth 1 https://github.com/tursodatabase/turso.git
cd turso
git apply ~/turso-sqlx-patch.patch
cargo build --release -p turso_sqlite3
```

## Verification

```
✅ cargo build — compiles on both branches
✅ cargo test — 1 test passes
✅ Server starts with fresh DB (Turso backend)
✅ Health endpoint responds: {"message":"server online"}
✅ User creation returns JWT tokens
✅ Watcher creation works
✅ Auth middleware blocks unauthenticated requests
✅ Worker runs without panics
✅ Graceful shutdown works
✅ ldd confirms: libsqlite3.so → libturso_sqlite3.so
```

## Outstanding Notes

- **Foreign keys are disabled** — Turso doesn't support `PRAGMA foreign_keys`. The `created_by REFERENCES users(id) ON DELETE CASCADE` constraint in the watchers table won't be enforced.
- **Turso is still WIP** — the Turso repo moves fast. When pulling upstream updates, re-apply the patches (or rebase the fork).
- **The `turso/` directory is gitignored** — the fork lives as a separate repo. The checkup repo only references it via the `libs/` symlinks.

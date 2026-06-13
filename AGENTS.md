# Simple, Data-Driven and Predictable

Use flat structures, avoid unecessary abstraction layers and trust sqlx and sqlite for the heavy lifting.   

Splitting domain and data access layer is ok, but the source of truth should always be the database; sqlite is the one choice exactly for testability.  


## API, project structure and handlers

- **Repository layer only when it pays off**: don't open a `*_repository.rs` file just to wrap a single SQL call. Repository modules are warranted only when a function is reused across multiple call sites (handlers, workers, tests) **or** when it isolates domain logic from the database schema (deduplication, status transitions, computed fields, fan-out). For everything else, inline `sqlx::query!` / `sqlx::query_as!` straight in the handler — the macro already gives you compile-time SQL checking.
- **DTO separation only when the shape actually differs**: a struct can carry `Serialize/Deserialize/Object/sqlx::FromRow` at the same time without becoming a problem — annotations are cheap. What you must avoid is **duplicated information scattered across the codebase** (the same field, transformation or invariant spelled out in two different files). The clearest case is the `User` feature: the create request receives a plaintext password, the row stores a bcrypt `passhash`, and the response must never leak a hash. That asymmetry — input ≠ storage ≠ output — is the only reason to keep separate DTOs (`CreateUserRequest`, the row, and `User`).
- **Macros for OpenAPI responses**: define precise `ApiResponse` enums using the `crate::api_response!` macro. It eliminates standard boilerplate while preserving accurate swagger/redoc schema generation.
- **Feature organization**: structure features inside `src/features/<feature>/` containing:
  - `mod.rs`: structs/DTOs definitions, OpenAPI routes delegation, and macro-based `ApiResponse` enums.
  - `<feature>_handlers.rs`: a single flat file containing all database operations and handlers logic.
- **Cross-feature infrastructure lives at the crate root**: anything that is not really a "feature" (auth, background workers, config) goes in `src/<module>.rs`, not inside `src/features/`. A worker that happens to read `Watcher` rows is not part of the `watchers` feature — it's runtime.

## Code style and preferences

- Avoid unecessary comments, left-over documentations and useless tests.  
- Prefer integration tests; test feature-wide, not serde's desserialization or sqlx mappings. The type system already enforces it.  
- Avoid writting numeric or pretty comments. Follow rust standard doc style and keep it simple.   

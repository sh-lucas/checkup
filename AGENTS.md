# Simple, Data-Driven and Predictable

Use flat structures, avoid unecessary abstraction layers and trust sqlx and sqlite for the heavy lifting.   

Splitting domain and data access layer is ok, but the source of truth should always be the database; sqlite is the one choice exactly for testability.  


## API, project structure and handlers

- **No necessity over Repository Layer**: Do not create database repository modules (e.g. `*_repository.rs`) when unecessary. Inlining SQL queries inside handler functions using `sqlx::query!` and `sqlx::query_as!` is fine to keep code flat and predictable.
- **DTO Separation**: Separate request inputs (e.g. `CreateUserRequest`) from response outputs (e.g. `User`) to prevent "annotation soup" and macro overload on domain structs.
- **Macros for OpenAPI responses**: Define precise `ApiResponse` enums using the `crate::api_response!` macro. It eliminates standard boilerplate while preserving accurate swagger/redoc schema generation.
- **Feature Organization**: Structure features inside `src/features/<feature>/` containing:
  - `mod.rs`: Structs/DTOs definitions, OpenAPI routes delegation, and macro-based `ApiResponse` enums.
  - `<feature>_handlers.rs`: A single flat file containing all database operations and handlers logic.

## Code style and preferences

- Avoid unecessary comments, left-over documentations and useless tests.  
- Prefer integration tests; test feature-wide, not serde's desserialization or sqlx mappings. The type system already enforces it.  
- Avoid writting numeric or pretty comments. Follow rust standard doc style and keep it simple.   
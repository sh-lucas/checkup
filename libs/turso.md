# Turso (Limbo) como backend SQLite do sqlx

## Contexto

O objetivo era substituir o SQLite embutido do sqlx pelo
[Turso Database](https://github.com/tursodatabase/turso) (anteriormente chamado Limbo), o
rewrite do SQLite em Rust feito pela Turso. A ideia: compilar o crate `turso_sqlite3` como
biblioteca nativa e enganar o `libsqlite3-sys` para linkar contra ela em vez do SQLite original.

---

## Estrutura do crate `turso_sqlite3`

O repositório do Turso expõe o crate `sqlite3/` com o seguinte `Cargo.toml`:

```toml
[lib]
crate-type = ["lib", "cdylib", "staticlib"]
```

Ele implementa a SQLite C API inteira usando `#[no_mangle] pub unsafe extern "C" fn sqlite3_*`,
tornando-a ABI-compatível com a `libsqlite3` original.

---

## Por que cdylib e não staticlib

A primeira tentativa foi linkar a `.a` (staticlib). Falhou com:

```
rust-lld: error: duplicate symbol: rust_eh_personality
```

Uma staticlib de um crate Rust **contém o runtime Rust inteiro** (panic handler, eh_personality,
alocador, etc.). Linkar duas staticlibs Rust no mesmo binário gera símbolo duplicado.

A **cdylib** resolve isso: ela encapsula o runtime internamente e expõe para o mundo externo
apenas os símbolos `#[no_mangle]` explícitos (`sqlite3_open`, `sqlite3_prepare_v2`, etc.),
exatamente como uma `libsqlite3.so` do sistema faria.

---

## Build

```bash
cd turso/
cargo build --release -p turso_sqlite3
```

Gera `turso/target/release/libturso_sqlite3.so`.

Para atualizar após pull do Turso, basta rodar o mesmo comando.

---

## Stubs adicionados

O sqlx-sqlite usa algumas funções que o Turso ainda não havia implementado. Foram adicionadas
ao final de `turso/sqlite3/src/lib.rs` como stubs com comportamento seguro (retornam `SQLITE_OK`,
`SQLITE_ERROR` ou ponteiro nulo conforme o caso):

- `sqlite3_update_hook`
- `sqlite3_commit_hook`
- `sqlite3_rollback_hook`
- `sqlite3_extended_result_codes`
- `sqlite3_load_extension`
- `sqlite3_unlock_notify`
- `sqlite3_sql`
- `sqlite3_column_database_name`
- `sqlite3_column_origin_name`
- `sqlite3_bind_blob64` (delega para `sqlite3_bind_blob`)
- `sqlite3_bind_text64` (delega para `sqlite3_bind_text`)

---

## Linkagem

### Symlinks em `libs/`

```
libs/libsqlite3.so  →  turso/target/release/libturso_sqlite3.so
libs/libsqlite3.a   →  turso/target/release/libturso_sqlite3.a   (não usado, mantido como referência)
```

### `libs/pkgconfig/sqlite3.pc`

O build script do `libsqlite3-sys` usa `pkg-config` para descobrir onde está a lib quando
`SQLITE3_LIB_DIR` é definido. O `.pc` customizado aponta para `libs/`:

```ini
libdir=/home/sh-lucas/Github/checkup/libs
includedir=/usr/include          # headers do sistema (libsqlite3-dev), usados pelo bindgen

Libs: -L${libdir} -lsqlite3
Libs.private: -lm -lz -ldl -lpthread
```

O bindgen usa o `sqlite3.h` do sistema (`/usr/include/sqlite3.h`) para gerar as bindings Rust,
mas o linker resolve `-lsqlite3` apontando para `libs/libsqlite3.so` (Turso).

### `.cargo/config.toml`

```toml
[env]
SQLITE3_LIB_DIR    = { value = "libs", relative = true }
SQLITE3_INCLUDE_DIR = "/usr/include"

[build]
rustflags = ["-C", "link-arg=-Wl,-rpath,$ORIGIN/../../libs"]
```

O `rpath` com `$ORIGIN` faz o binário encontrar `libs/libsqlite3.so` em runtime sem precisar
instalar nada no sistema. `$ORIGIN` resolve para o diretório do binário
(`target/debug/` ou `target/release/`), e `../../libs` sobe dois níveis até a raiz do projeto.

Para confirmar o resultado:

```bash
ldd target/debug/checkup | grep sqlite
# libsqlite3.so => .../checkup/target/debug/../../libs/libsqlite3.so

readelf -d target/debug/checkup | grep RUNPATH
# RUNPATH: [$ORIGIN/../../libs]
```

### Feature do sqlx

```toml
# Cargo.toml
sqlx = { ..., features = ["sqlite-unbundled", ...] }
```

`sqlite-unbundled` instrui o sqlx a não embutir o SQLite e usar o `libsqlite3-sys` sem bundling,
que por sua vez respeita as variáveis de ambiente acima.

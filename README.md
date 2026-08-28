# checkup

API de monitoramento escrita em Rust para acompanhar a saúde do servidor e de
serviços externos. Ela coleta métricas da máquina em background, registra o
resultado de verificações HTTP e expõe os dados por REST e Server-Sent Events
(SSE).

O projeto [lucas-schwalm-silva](https://github.com/lucas-schwalm-silva) usa o
check.up como backend do painel de estatísticas em tempo real da máquina. O
frontend pode consumir `/api/metrics` para uma leitura pontual ou
`/api/metrics/stream` para receber atualizações continuamente.

## O que a API oferece

- métricas de CPU, load average, memória, disco, I/O, pressão de CPU e uptime;
- stream SSE de métricas atualizado em tempo real;
- cadastro, autenticação por JWT e consulta do usuário autenticado;
- criação e listagem de URLs monitoradas (watchers);
- consulta dos pings que ficaram offline;
- documentação OpenAPI em `/docs` e `/redoc`.

As métricas são coletadas a partir das interfaces do Linux (`/proc` e
`statvfs`), portanto a execução é voltada principalmente para Linux. Os
workers verificam os watchers periodicamente, registram os pings e removem
histórico com mais de sete dias.

## Executando localmente

Requisitos: Rust, SQLite e um ambiente Linux.

```bash
export PORT=3000
export DATABASE_URL=sqlite://database/database.db
export JWT_SECRET='change-me'
cargo run
```

As migrações do SQLx são executadas automaticamente na inicialização. Os
parâmetros opcionais são `PING_INTERVAL_SECS`, `NUM_PING_WORKERS`,
`SLOW_QUERY_THRESHOLD_MS`, `OTEL_SERVICE_NAME`, `OTEL_SERVICE_VERSION`,
`DEPLOYMENT_ENVIRONMENT` e `OTEL_EXPORTER_OTLP_ENDPOINT`.

Não existem credenciais padrão. O endpoint de cadastro exige um JWT válido;
isso permite que o bootstrap de usuários seja controlado pelo ambiente de
deploy. Em instalações sem usuários, é necessário provisionar o primeiro
acesso por um fluxo administrativo seguro antes de usar a API autenticada.

## Principais endpoints

| Método | Endpoint | Descrição |
| --- | --- | --- |
| GET | `/api/` | Health check |
| GET | `/api/metrics` | Métricas atuais |
| GET | `/api/metrics/stream` | Métricas via SSE |
| POST | `/api/users/login` | Gera um JWT |
| POST | `/api/users/register` | Cria usuário autenticado |
| GET | `/api/users/me` | Perfil do usuário autenticado |
| POST | `/api/watchers/create` | Cria URL monitorada |
| GET | `/api/watchers/list` | Lista URLs do usuário |
| GET | `/api/pings/down` | Lista pings offline |

Os endpoints protegidos usam `Authorization: Bearer <token>`. A especificação
completa e os schemas ficam disponíveis no Swagger UI e no ReDoc.

## Desenvolvimento e testes

O desenvolvimento prioriza uma estrutura simples e orientada a dados: SQLite
é a fonte de verdade, as mutações passam pelo SQLx e cada feature mantém suas
rotas, modelos e operações próximas umas das outras.

Os testes priorizam fluxos integrados com SQLite em memória e clientes HTTP
de teste, complementados por testes unitários para regras isoladas. Para
validar as alterações:

```bash
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

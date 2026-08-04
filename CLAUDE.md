# Tenet

Rust workspace for reverse engineering a target's surface. Websites today, mobile binaries next.
A gateway accepts scans over HTTP, an analyzer claims them from Postgres and runs the pipeline.

## Commands

```bash
docker compose up -d                                    # postgres
cargo build --workspace
./scripts/check-conventions.sh                          # file length, no comments, layer boundaries
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings   # CI gate: warnings are errors
cargo run -p tenet-gateway                              # or -analyzer
```

All four run in CI in that order. Lint levels live in `[workspace.lints]` in the root `Cargo.toml`
and every member opts in with `[lints] workspace = true` — a new crate without that stanza is
silently unlinted. Thresholds are in `clippy.toml`, formatting in `rustfmt.toml`.

Migrations in `migrations/` run automatically on `tenet-gateway` startup via `sqlx::migrate!`.
The analyzer never migrates; it expects the gateway to have run first.

## Layout

Every workspace member lives at the repo root and is named with the `tenet-` prefix. Crate names
use the same prefix (`tenet_types` when imported). New members must follow the prefix and be added
to both `members` and `workspace.dependencies` in the root `Cargo.toml`.

Shared crates: `tenet-types` (`TargetKind`, `ScanStatus`, `Finding`, `Endpoint`, `AuthScheme`,
response envelopes), `tenet-errors` (`AppError` + `IntoResponse`), `tenet-config` (env loading,
tracing init, shutdown token), `tenet-database` (pool), `tenet-web` (the website engine),
`tenet-spec` (OpenAPI generation), `tenet-mobile` (binary analysis seam).

Services: `tenet-gateway` (HTTP 8080), `tenet-analyzer` (no listener, polls Postgres).

`tenet-web` is pure: it takes fetched bytes and returns findings and endpoints. It never opens a
socket. Fetching belongs to the analyzer's infrastructure, which keeps the engine testable without
a network and keeps signatures cheap to add.

## Deployment

One root `Dockerfile` builds every service — `docker build --build-arg SERVICE=<name> .` selects
the binary. Replicas are interchangeable: the analyzer claims work with
`FOR UPDATE SKIP LOCKED`, so no replica owns a partition and none keeps state between ticks.
Anything a new service adds must preserve that — no per-replica sharding, shared state in Postgres.

## Clean Architecture

Each service is `domain/` → `application/` → `infrastructure/`, with `main.rs` as the only
composition root.

- `domain/` — entities and port traits. Depends on shared crates only. No sqlx queries, no axum,
  no reqwest, no scraper. Ports are `#[async_trait] pub trait X: Send + Sync` returning
  `Result<T, AppError>`.
- `application/` — one use case per file, named after it (`submit_scan.rs` → `SubmitScan`). Struct
  holds `Arc<dyn Port>` fields, exposes `new(...)` and a single `execute(...)`. Business rules live
  here (target validation, script budget clamping, the retry decision, spec assembly).
- `infrastructure/` — adapters implementing the ports: `http/` (routes, handlers, dto, views, auth),
  `persistence/` (postgres repositories, SQL in `statements.rs`), `fetch/` (the reqwest client).
- `state.rs` — `AppState` for axum services: `Arc<UseCase>` fields plus config values.

Dependencies point inward. Infrastructure knows about domain and application; never the reverse.
Wiring — concrete adapters constructed and injected as `Arc<dyn Port>` — happens only in `main.rs`.

An analyzer that implements a port from its own application layer is fine (`AnalyzeWebTarget` is a
`TargetAnalyzer`) because it depends only on other ports. An adapter that needs a crate the
application layer may not touch belongs in `infrastructure` — that is why hashing lives in
`ReqwestPageFetcher` and not in the use case.

## Conventions

- **Max 200 lines per file.** Split by responsibility when a file approaches it, never by arbitrary
  cut. A use case that outgrows 200 lines usually hides a second use case. Enforced by
  `scripts/check-conventions.sh`.
- **No comments.** No `//`, no `///` doc comments, no `#[doc]`. Names, types, and small functions
  carry the meaning. If a block needs a comment, extract it into a named function. Remove comments
  from any file you touch. Enforced by `scripts/check-conventions.sh`; the clippy config allows the
  `missing_*_doc` lints so nothing ever demands a doc comment back.
- **Clean code.** Small functions, one job each. Early returns over nesting. No dead code, no
  speculative abstraction, no `unwrap()`/`expect()` outside `main.rs` startup. A `LazyLock<Regex>`
  holds `Option<Regex>` and callers `let ... else { return }` rather than unwrap.
- Errors: return `AppError` everywhere. Add `From<X> for AppError` in `tenet-errors` rather than
  mapping the same conversion in several call sites. `AppError::is_transient` drives the retry
  ladder — a new variant must decide whether it is worth retrying.
- HTTP boundary: request DTOs in `infrastructure/http/dto.rs`, read models in `views.rs`, both
  converted with `From` impls. Handlers stay thin — parse, call `execute`, map to DTO.
- Config: every setting goes through `tenet_config::Config::from_env()` with a default matching
  `.env.example`. Never read `std::env` from a service.
- Logging: `tracing` with structured fields (`tracing::info!(scan_id = %id, "message")`),
  lowercase messages.
- Naming: modules and files `snake_case`, adapters named `<Tech><Port>` (`PostgresScanRepository`,
  `ReqwestPageFetcher`, `PostgresAnalysisWriter`).
- Async: `tokio` runtime, `Arc` for shared state, no blocking calls in async fns.
  `std::sync::Mutex`, `std::sync::RwLock`, and `std::thread::sleep` are denied by clippy — use the
  `tokio` equivalents. `scraper::Html` is not `Send`; keep it inside a sync function so it never
  crosses an `.await`.
- Tests: pure functions carry `#[cfg(test)] mod tests` in the same file. Name a test after the
  behaviour it pins (`a_static_asset_is_not_an_endpoint`), not after the function it calls.
- Imports: `std` first, then external crates, then `crate::`, each group separated by a blank line
  and one `use` per module. Stable rustfmt cannot enforce this (`group_imports` and
  `imports_granularity` are nightly-only), so it is a review convention.

## Scan flow

`gateway` validates the target and inserts a `scans` row as `queued` → `analyzer` claims a batch
with `FOR UPDATE SKIP LOCKED`, marking rows `running` and incrementing `attempts` → the web
analyzer fetches the page, harvests up to `max_scripts` script assets plus every inline script,
then runs `tenet-web`: `detect` (fingerprints), `endpoints` (method calls, fetch calls, path
literals), `auth_findings` (challenge headers, session cookies, source markers, auth-shaped paths)
→ artifacts, findings and endpoints are written in one transaction and the scan is marked
`succeeded`.

A transient failure with attempts left goes back to `queued`; anything else is `failed` with the
error recorded on the row. Only `AppError::Unavailable` is transient.

Findings and endpoints are deduped by identity, keeping the highest confidence — `Finding::identity`
is `kind:name:value` and `Endpoint::identity` is `method path`. The unique constraints in
`migrations/` mirror those identities, so a re-scan is idempotent.

Mobile targets route to `tenet-mobile`, whose `PendingBinaryAnalyzer` reports that binary analysis
is unavailable. Implementing `BinaryAnalyzer` for real is the only change needed to light that path
up — the queue, retry ladder, storage and read API already handle it.

# Tenet

Rust workspace for reverse engineering a target's surface. Websites today, mobile binaries next.
A gateway accepts scans over HTTP, an analyzer claims them from Postgres and runs the pipeline,
either by fetching the served HTML or by driving Chromium and watching what the app calls.

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

The analyzer wants Chromium for `engine=browser` scans. It finds a local binary, honours
`CHROME_BIN`, or connects to an existing DevTools endpoint via `CHROME_WS_URL` — the last one is
the easiest way to develop against a browser in a container.

## Layout

Every workspace member lives at the repo root and is named with the `tenet-` prefix. Crate names
use the same prefix (`tenet_types` when imported). New members must follow the prefix and be added
to both `members` and `workspace.dependencies` in the root `Cargo.toml`.

Shared crates: `tenet-types` (`TargetKind`, `Engine`, `ScanStatus`, `Finding`, `Endpoint`,
`AuthScheme`, response envelopes), `tenet-errors` (`AppError` + `IntoResponse`), `tenet-config`
(env loading, tracing init, shutdown token), `tenet-database` (pool), `tenet-web` (the static
website engine), `tenet-stealth` (fingerprint normalization + challenge detection),
`tenet-browser` (the Chromium driver), `tenet-spec` (OpenAPI generation), `tenet-mobile`
(binary analysis seam).

Services: `tenet-gateway` (HTTP 8080), `tenet-analyzer` (no listener, polls Postgres).

`tenet-web` is pure: it takes fetched bytes and returns findings and endpoints. It never opens a
socket. Fetching belongs to the analyzer's infrastructure, which keeps the engine testable without
a network and keeps signatures cheap to add.

`tenet-browser` is the opposite — it exists to talk to Chromium and does nothing else. It renders a
url and reports what it saw (`RenderedPage`: html, the main response, captured requests, storage
keys). It does no analysis, so everything it returns flows through the same `tenet-web` core.
Only the analyzer's `infrastructure/browser/` may depend on it.

`tenet-stealth` is pure and has zero dependencies. It produces plain data — launch args, an
injectable init script, a `StealthProfile` (user agent, client hints, languages, webgl strings)
derived from the browser's real user agent, a deterministic settle-delay jitter, and
`detect_challenge` — and never touches chromiumoxide. `tenet-browser` translates a `StealthProfile`
into CDP calls in `stealth_apply.rs`; the analyzer maps a detected challenge to a finding. Keeping
it dependency-free is deliberate: the whole crate is unit-testable without a browser.

## Deployment

One root `Dockerfile` builds every service — `docker build --build-arg SERVICE=<name> .` selects
the binary. Replicas are interchangeable: the analyzer claims work with
`FOR UPDATE SKIP LOCKED`, so no replica owns a partition and none keeps state between ticks.
Anything a new service adds must preserve that — no per-replica sharding, shared state in Postgres.

## Clean Architecture

Each service is `domain/` → `application/` → `infrastructure/`, with `main.rs` as the only
composition root.

- `domain/` — entities and port traits. Depends on shared crates only. No sqlx queries, no axum,
  no reqwest, no scraper, no chromiumoxide, no `tenet_browser`. Ports are
  `#[async_trait] pub trait X: Send + Sync` returning `Result<T, AppError>`.
- `application/` — one use case per file, named after it (`submit_scan.rs` → `SubmitScan`). Struct
  holds `Arc<dyn Port>` fields, exposes `new(...)` and a single `execute(...)`. Business rules live
  here (target validation, script budget clamping, the retry decision, spec assembly).
- `infrastructure/` — adapters implementing the ports: `http/` (routes, handlers, dto, views, auth),
  `persistence/` (postgres repositories, SQL in `statements.rs`), `fetch/` (the reqwest client),
  `browser/` (the `tenet-browser` adapter).
- `state.rs` — `AppState` for axum services: `Arc<UseCase>` fields plus config values.

Dependencies point inward. Infrastructure knows about domain and application; never the reverse.
Wiring — concrete adapters constructed and injected as `Arc<dyn Port>` — happens only in `main.rs`.

An analyzer that implements a port from its own application layer is fine (`AnalyzeWebTarget` is a
`TargetAnalyzer`) because it depends only on other ports. An adapter that needs a crate the
application layer may not touch belongs in `infrastructure` — that is why hashing lives in
`ReqwestPageFetcher` and `ChromiumPageRenderer` rather than in a use case, and why both return a
`FetchedDocument` that already carries its `sha256` and `byte_size`.

Both web engines share one analysis core. `web_analysis::analyse` takes a page, its scripts, and
any runtime observations, and returns the `Analysis`; `AnalyzeWebTarget` passes no observations
and `AnalyzeRenderedTarget` passes what the browser saw. A new engine should add observations to
that call, never fork the analysis.

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
- Chromium: `Browser::launch` and `Browser::connect` both hand back a `Handler` that must be
  pumped on its own task or every command hangs. `Session` owns that task and flips an `alive`
  flag when the stream ends, which is how `ChromiumRenderer` notices a dead browser and relaunches.
  Event listeners are aborted on `Drop` — a `NetworkCapture` that outlives its page leaks a task.
- Launch args: chromiumoxide's `Arg` stores the string verbatim and renders it as `--{arg}`, so a
  flag must be passed WITHOUT its leading dashes (`disable-dev-shm-usage`, not
  `--disable-dev-shm-usage`) or it reaches Chrome as `----disable-dev-shm-usage`. `STEALTH_ARGS`
  keeps this convention. Launch args only apply to a browser Tenet launches, never to one reached
  through `CHROME_WS_URL`; per-page masking (UA override, init script) applies to both.
- Loop bodies inside a spawned task hit the nesting limit fast. Extract the body into a named
  function (`pump_handler`, `store_main_response`) rather than reaching for an `allow`.
- Tests: pure functions carry `#[cfg(test)] mod tests` in the same file. Name a test after the
  behaviour it pins (`a_static_asset_is_not_an_endpoint`), not after the function it calls.
- Imports: `std` first, then external crates, then `crate::`, each group separated by a blank line
  and one `use` per module. Stable rustfmt cannot enforce this (`group_imports` and
  `imports_granularity` are nightly-only), so it is a review convention.

## Scan flow

`gateway` validates the target and inserts a `scans` row as `queued` → `analyzer` claims a batch
with `FOR UPDATE SKIP LOCKED`, marking rows `running` and incrementing `attempts` → `AnalyzeScan`
picks an analyzer by `kind` then `engine` → the chosen analyzer produces the page and its scripts,
then runs `tenet-web`: `detect` (fingerprints), `endpoints` (method calls, fetch calls, path
literals), `auth_findings` (challenge headers, session cookies, source markers, auth-shaped paths)
→ artifacts, findings and endpoints are written in one transaction and the scan is marked
`succeeded`.

`engine=http` fetches the served HTML and harvests up to `max_scripts` linked scripts plus every
inline script. `engine=browser` renders the page in Chromium, harvests scripts from the hydrated
DOM the same way, and adds what it observed: XHR and fetch calls become endpoints at `0.95` with
source `runtime`, an `Authorization` header seen on the wire pins the scheme at `0.95`, and
`localStorage` keys that look like tokens are reported under their real names. Observations are
merged with static findings by the usual identity-and-confidence rule, so the browser engine only
ever adds or outranks.

The browser engine is optional. If `BROWSER_ENABLED=0`, or Chromium cannot be reached, the
analyzer still serves `http` scans and wires an `UnavailableTarget` in place of the renderer, so
`engine=browser` scans fail with a clear reason instead of silently downgrading to a weaker scan.

Stealth (`BROWSER_STEALTH=1`, on by default) normalizes the fingerprint anti-bot uses to single out
automation: it applies `tenet-stealth`'s launch args, overrides the user agent and client hints to
match a real browser, and injects an init script that masks `navigator.webdriver` (to `false`, the
value a genuine Chrome reports — not `undefined`), languages, plugins, vendor and the WebGL
vendor/renderer. `BROWSER_REGION` (e.g. `id`) sets a plausible `Accept-Language` and
`navigator.languages`. Whether a target's protection is actually defeated is target-specific;
stealth normalizes the browser, it does not solve CAPTCHAs. `detect_challenge` runs on every
rendered page and records a `bot-protection` finding when the target served an interstitial, so a
thin scan is explained rather than mistaken for a site with no API. Only scan targets you own or
are authorised to assess, and respect their rate limits.

A transient failure with attempts left goes back to `queued`; anything else is `failed` with the
error recorded on the row. Only `AppError::Unavailable` is transient.

Findings and endpoints are deduped by identity, keeping the highest confidence — `Finding::identity`
is `kind:name:value` and `Endpoint::identity` is `method path`. The unique constraints in
`migrations/` mirror those identities, so a re-scan is idempotent.

Mobile targets route to `tenet-mobile`, whose `PendingBinaryAnalyzer` reports that binary analysis
is unavailable. Implementing `BinaryAnalyzer` for real is the only change needed to light that path
up — the queue, retry ladder, storage and read API already handle it.

# Tenet

A reverse engineering framework for understanding what an application is actually built on and
what it actually talks to. Point it at a website and it returns the technology stack, the private
API it calls, how that API is authenticated, and an OpenAPI document you can hand to a client
generator. It can read the served HTML, or drive a real browser and watch the calls the app makes
after it hydrates. Mobile binaries are next.

Tenet only reads what a target already serves to any browser. Use it on systems you own or are
authorised to assess.

## What a scan produces

| Output | What it is |
| --- | --- |
| Findings (`technology`) | Framework, CDN, WAF, server, analytics and vendor detections with confidence and the evidence that triggered them |
| Findings (`auth`) | Detected auth schemes, session cookies, token storage, and the endpoints that look like login or token exchange |
| Endpoints | Method + path, templated (`/api/v1/users/{userId}`), with an origin when the call was absolute, plus the artifact it came from |
| Observed calls | With `engine: browser`, every XHR and fetch the app actually issues — recorded at confidence 0.95 with the verb the browser really used, and marked `gated` when the gateway answered 401/403/429 |
| Anti-bot | Detected protection vendor (DataDome, Akamai, PerimeterX, Kasada, Shopee's `shpsec`/`antifraudivs`), and a challenge finding when a scan was interstitialed |
| WASM analysis | Any WebAssembly a page loads — a separate `.wasm` file, or one hidden as base64 inside a JS bundle — reverse-engineered: host imports, exports, embedded strings, toolchain, and whether it looks like a fingerprint/anti-fraud module |
| OpenAPI | A 3.1 document assembled from the endpoints and auth schemes, with path parameters and per-operation confidence |
| Artifacts | Every fetched document with its sha256 and size, so a re-scan can be compared against the last one |

## Quickstart

```bash
docker compose up -d                 # postgres on 5432
cp .env.example .env

cargo run -p tenet-gateway           # http api on 8080, runs migrations
cargo run -p tenet-analyzer          # in a second shell, polls for queued scans
```

Submit a scan and read it back:

```bash
curl -X POST localhost:8080/v1/scans \
  -H 'x-api-key: dev-key' -H 'content-type: application/json' \
  -d '{"target":"https://example.com","kind":"web","engine":"browser"}'

SCAN=<scan_id from the response>
curl localhost:8080/v1/scans/$SCAN           -H 'x-api-key: dev-key'
curl localhost:8080/v1/scans/$SCAN/findings  -H 'x-api-key: dev-key'
curl localhost:8080/v1/scans/$SCAN/endpoints -H 'x-api-key: dev-key'
curl localhost:8080/v1/scans/$SCAN/openapi   -H 'x-api-key: dev-key' > api.json
```

Swagger UI is at `localhost:8080/docs`. Every route except `/healthz` and the docs wants
`x-api-key` (or `Authorization: Bearer <key>`), matched against `API_KEY`.

`scripts/sample-scan.sh <target> <engine>` does all of the above in one go and prints a readable
report. [`examples/shopee.md`](examples/shopee.md) is a real run against a production SPA — 1
endpoint with `http`, 58 with `browser`. [`examples/anti-bot.md`](examples/anti-bot.md) is an
evidence-based teardown of Shopee's anti-bot and the capabilities it drove into the tool.

## API

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/healthz` | liveness, unauthenticated |
| `POST` | `/v1/scans` | queue a scan — `target`, `kind` (`web` \| `mobile`), `engine` (`http` \| `browser`), optional `max_scripts` |
| `GET` | `/v1/scans/{id}` | status, attempts, timings, error |
| `GET` | `/v1/scans/{id}/findings` | technology and auth findings |
| `GET` | `/v1/scans/{id}/endpoints` | discovered endpoints |
| `GET` | `/v1/scans/{id}/openapi` | OpenAPI 3.1 document |

## How it works

```
tenet-gateway ──insert queued scan──> postgres <──claim (SKIP LOCKED)── tenet-analyzer
     ^                                    ^                                   │
     └────── read findings/spec ──────────┘                    engine=http ───┤
                                                                              │
                                            fetch page ──> harvest scripts ───┤
                                                                              │
                                                            engine=browser ───┤
                                                                              │
                          render + capture XHR/fetch ──> harvest scripts ─────┤
                                                                              v
                                          detect / endpoints / auth_findings  tenet-web
```

`tenet-web` is a pure library: give it fetched bytes and it hands back findings and endpoints. It
never opens a socket, which is why its tests run without a network. Both engines feed the same
analysis core, so the browser path adds observations rather than replacing anything.

### The two engines

`engine: http` fetches the served HTML and its linked scripts. Fast, no browser, and enough for
server-rendered sites.

`engine: browser` drives real Chromium, waits for the app to hydrate, and records every XHR and
fetch it issues. That catches what static reading cannot: URLs assembled at runtime, verbs chosen
from variables, and calls made only after login or interaction. Observed calls land at confidence
`0.95` with source `runtime`; an `Authorization` header seen on a live request pins the auth scheme
at `0.95`, and `localStorage` keys are reported by their real names.

Chromium is found automatically, or set `CHROME_BIN`. To use a browser you already run, set
`CHROME_WS_URL` to its DevTools endpoint and Tenet connects instead of launching. If neither is
available the analyzer still starts and serves `http` scans; `browser` scans fail with a clear
message. `BROWSER_ENABLED=0` turns the engine off outright.

### Stealth

Anti-bot systems single out automation by its fingerprint — the `navigator.webdriver` flag, a
headless user agent, missing plugins, a giveaway WebGL renderer. `tenet-stealth` normalizes that
fingerprint so a `browser` scan presents like an ordinary visitor: real user agent and client
hints, `navigator.webdriver` reporting `false` (the value a genuine Chrome returns), plausible
languages, plugins, vendor and WebGL strings, and a small randomized settle delay. It is on by
default; `BROWSER_STEALTH=0` disables it and `BROWSER_REGION=id` sets a country-appropriate
`Accept-Language`.

With `BROWSER_BEHAVIOR=1` (default) it also plays a human-like interaction pass — mouse moves,
scrolls and dwells through Chrome's real input pipeline — because a session with zero mouse or scroll
events is the classic bot tell (measured: `moves_0/wheels_0` becomes `moves_19/wheels_12`).

This normalizes the browser — it does not solve CAPTCHAs or defeat every protection. When a target
serves a challenge anyway, Tenet detects the interstitial and records a `bot-protection` finding, so
a thin result is explained rather than mistaken for a site with no API. The `http` engine's TLS
fingerprint is still not Chrome's, IP reputation is unaddressed, and a server-side risk engine can
still win — [`examples/automation-detection.md`](examples/automation-detection.md) maps the six
detection layers, which stealth reaches, and which it does not. Use it only on targets you own or
are authorised to assess, and respect their rate limits.

Endpoint extraction reads four signals and keeps the strongest per `method path`: observed runtime
calls (0.95, browser engine only), method calls (`axios.post("/api/v1/sessions")` → 0.85), fetch
calls (0.7), and bare path literals matching `/api`, `/rest`, `/graphql`, `/gateway` or `/v1`
(0.45). Template segments are normalised — `:userId` and `${userId}` both become `{userId}` — and
static assets are discarded.

A call argument only counts when the string literal is the whole argument, so `fetch("/ap" + rest)`
does not invent an `/ap` endpoint. A bare literal is still recorded even when it is concatenated,
because a path constant is real evidence — that is what the 0.45 confidence is for.

### Crates

| Crate | Role |
| --- | --- |
| `tenet-types` | Shared vocabulary: `TargetKind`, `ScanStatus`, `Finding`, `Endpoint`, `AuthScheme` |
| `tenet-errors` | `AppError` with `IntoResponse` and `From` conversions |
| `tenet-config` | `Config::from_env()`, tracing init, shutdown token |
| `tenet-database` | Postgres pool |
| `tenet-web` | Fingerprints, script harvesting, endpoint and auth extraction |
| `tenet-stealth` | Fingerprint normalization, launch args, challenge detection (pure, no deps) |
| `tenet-wasm` | WebAssembly reverse engineering: imports, exports, strings, toolchain, signals |
| `tenet-browser` | Chromium driver: renders a page and captures its live requests |
| `tenet-spec` | OpenAPI 3.1 generation |
| `tenet-mobile` | `BinaryAnalyzer` port and the pending implementation |
| `tenet-gateway` | HTTP API |
| `tenet-analyzer` | Claim loop and analysis pipeline |

## Adding a fingerprint

Signatures are one line each in `tenet-web/src/fingerprint/signatures.rs`. Needles are matched
lowercase; an empty needle means "this header exists at all".

```rust
head("x-vercel-id", "", "Vercel", "hosting", 0.95),
cookie("laravel_session", "Laravel", "backend", 0.9),
body("/_next/static", "Next.js", "frontend", 0.95),
script("js.stripe.com", "Stripe", "payments", 0.95),
```

## Contributing

`CLAUDE.md` is the working agreement: clean architecture per service, max 200 lines per file, no
comments, no `unwrap()` outside startup. Before opening a PR:

```bash
./scripts/check-conventions.sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Roadmap

- **Mobile binaries.** `tenet-mobile` already defines the `BinaryAnalyzer` port and routes mobile
  scans to it; APK and IPA unpacking, string and manifest extraction are the missing implementation.
- **Secret detection.** `FindingKind::Secret` exists and is unused — API keys and tokens left in
  bundles are the obvious next finding type.
- **Request shapes.** Endpoints carry a method and a path; bodies, query parameters and response
  schemas would make the generated spec directly usable. The browser engine already sees real
  request bodies, so this is mostly a matter of recording them.
- **Authenticated scanning.** The browser engine renders as an anonymous visitor. Driving a login
  first would expose the endpoints that only exist behind a session.
- **Getting past a challenge.** Stealth normalizes the fingerprint but does not solve an
  interstitial. Waiting out a JS challenge, or wiring a solver, would turn a detected
  `bot-protection` finding into a completed scan.
- **Deobfuscating WASM sensors.** `tenet-wasm` recovers a module's imports, exports, strings and
  toolchain and flags anti-fraud behaviour, but stops short of lifting the token algorithm out of
  the bytecode. Disassembly to WAT and dataflow over the fingerprint routine is the next depth.
- **Telling an API from a CDN payload.** Every observed `fetch` is recorded, so content assets
  pulled at runtime sit alongside real API calls — on the Shopee sample that is 24 rows of noise
  against 21 useful ones. Classifying by response content type or origin role would fix it.

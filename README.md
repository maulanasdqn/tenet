# Tenet

A reverse engineering framework for understanding what an application is actually built on and
what it actually talks to. Point it at a website and it returns the technology stack, the private
API it calls, how that API is authenticated, and an OpenAPI document you can hand to a client
generator. Mobile binaries are next.

Tenet only reads what a target already serves to any browser. Use it on systems you own or are
authorised to assess.

## What a scan produces

| Output | What it is |
| --- | --- |
| Findings (`technology`) | Framework, CDN, WAF, server, analytics and vendor detections with confidence and the evidence that triggered them |
| Findings (`auth`) | Detected auth schemes, session cookies, token storage, and the endpoints that look like login or token exchange |
| Endpoints | Method + path, templated (`/api/v1/users/{userId}`), with an origin when the call was absolute, plus the artifact it came from |
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
  -d '{"target":"https://example.com","kind":"web"}'

SCAN=<scan_id from the response>
curl localhost:8080/v1/scans/$SCAN           -H 'x-api-key: dev-key'
curl localhost:8080/v1/scans/$SCAN/findings  -H 'x-api-key: dev-key'
curl localhost:8080/v1/scans/$SCAN/endpoints -H 'x-api-key: dev-key'
curl localhost:8080/v1/scans/$SCAN/openapi   -H 'x-api-key: dev-key' > api.json
```

Swagger UI is at `localhost:8080/docs`. Every route except `/healthz` and the docs wants
`x-api-key` (or `Authorization: Bearer <key>`), matched against `API_KEY`.

## API

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/healthz` | liveness, unauthenticated |
| `POST` | `/v1/scans` | queue a scan — `target`, `kind` (`web` \| `mobile`), optional `max_scripts` |
| `GET` | `/v1/scans/{id}` | status, attempts, timings, error |
| `GET` | `/v1/scans/{id}/findings` | technology and auth findings |
| `GET` | `/v1/scans/{id}/endpoints` | discovered endpoints |
| `GET` | `/v1/scans/{id}/openapi` | OpenAPI 3.1 document |

## How it works

```
tenet-gateway ──insert queued scan──> postgres <──claim (SKIP LOCKED)── tenet-analyzer
     ^                                    ^                                   │
     └────── read findings/spec ──────────┘                                   │
                                                                              v
                                            fetch page ──> harvest scripts ──> tenet-web
                                                                              │
                                          detect / endpoints / auth_findings <┘
```

`tenet-web` is a pure library: give it fetched bytes and it hands back findings and endpoints. It
never opens a socket, which is why its 37 tests run without a network. The analyzer's
infrastructure owns fetching, hashing and persistence.

Endpoint extraction reads three signals and keeps the strongest per `method path`: method calls
(`axios.post("/api/v1/sessions")` → confidence 0.85), fetch calls (0.7), and bare path literals
matching `/api`, `/rest`, `/graphql`, `/gateway` or `/v1` (0.45). Template segments are normalised
— `:userId` and `${userId}` both become `{userId}` — and static assets are discarded.

### Crates

| Crate | Role |
| --- | --- |
| `tenet-types` | Shared vocabulary: `TargetKind`, `ScanStatus`, `Finding`, `Endpoint`, `AuthScheme` |
| `tenet-errors` | `AppError` with `IntoResponse` and `From` conversions |
| `tenet-config` | `Config::from_env()`, tracing init, shutdown token |
| `tenet-database` | Postgres pool |
| `tenet-web` | Fingerprints, script harvesting, endpoint and auth extraction |
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
- **Rendered scanning.** Only served HTML and linked scripts are read today, so endpoints reachable
  solely through a rendered SPA are missed. A headless engine would close that gap.
- **Request shapes.** Endpoints carry a method and a path; bodies, query parameters and response
  schemas would make the generated spec directly usable.

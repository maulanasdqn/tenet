# tenet-ui

A web UI for driving Tenet, so you can submit scans and read findings without `curl`. Vite + React +
TypeScript, TanStack Router / Query / Table, and shadcn-style components on Tailwind v4.

## Run

```bash
pnpm install
pnpm dev          # http://localhost:5173
```

It talks to `tenet-gateway` over HTTP. The gateway sends permissive CORS, so the SPA can call it from
any origin. Set the gateway URL and API key from the **Settings** button in the header (defaults:
`http://127.0.0.1:8080`, `dev-key`, stored in `localStorage`).

Bring the backend up first:

```bash
docker compose up -d
cargo run -p tenet-gateway        # HTTP 8080
cargo run -p tenet-analyzer       # polls for scans
```

## What it does

- **Queue a scan** — target, kind (`web` / `mobile`), engine (`http` / `browser`), script budget.
- **Recent scans** — kept in the browser (there is no list endpoint on the gateway), each polling its
  own status with TanStack Query until it settles.
- **Scan detail** — live status, then three tables:
  - **Findings** — technology, auth, gated endpoints, WASM/native anti-fraud (TanStack Table, sortable
    and filterable).
  - **Endpoints** — method, path, origin, `runtime` vs `static`, confidence.
  - **OpenAPI** — the generated 3.1 document, viewable and downloadable.

## Layout

```
src/
  lib/          api client, types, localStorage settings
  components/
    ui/         shadcn-style primitives (button, input, card, table, badge, …)
    data-table.tsx        generic TanStack Table
    scan-form.tsx         queue a scan (TanStack Query mutation)
    recent-scans.tsx      localStorage list, live status
    findings-table.tsx    findings query + columns
    endpoints-table.tsx   endpoints query + columns
    openapi-view.tsx      spec viewer + download
    scan-detail.tsx       status + tabs
  routes/       root layout, index (dashboard), scans/$scanId
  router.tsx    TanStack Router route tree
  main.tsx      QueryClient + Router providers
```

## Build

```bash
pnpm build        # tsc -b && vite build -> dist/
```

`dist/` is static; serve it from anywhere (the gateway's CORS lets it reach the API cross-origin).

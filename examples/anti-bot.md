# How Shopee's anti-bot works, and what Tenet learned from it

This is an evidence-based teardown of `shopee.co.id`'s bot protection, gathered with Tenet plus
`curl`, and the three capabilities it drove into the tool. Everything here comes from content the
site serves to any browser. Study or harden systems you own or are authorised to assess.

## What was measured

| Probe | Result | Meaning |
| --- | --- | --- |
| `curl -I https://shopee.co.id/` | `server: SGW`, no `Set-Cookie`, no Cloudflare/Akamai | The edge is Shopee's own gateway; device tokens are minted by JavaScript, not the server |
| `GET /api/v4/pages/get_category_tree` | `200`, real JSON | Low-value catalog reads are open |
| `GET /api/v4/recommend/recommend` | `403` `{"error":90309999,"action_type":2,"redirect_to_error_page":true}` | Personalized endpoints are gated at the gateway |
| `POST /api/v4/abtest/traffic/get_web_experiments` | `200`, contains `"dimension":"deviceFingerprint"` | Decisions key on a device fingerprint |
| Homepage shell | loads `shopee-trackingsdk` + `antifraudivs/*` bundles, sets `SPC_F` | A client-side security SDK fingerprints the device and mints the token |
| Browser scan | `POST df.infra.sz.shopee.co.id/v2/shpsec/web/report` | The SDK reports the fingerprint to "shpsec" (Shopee security) |

## The four-layer model

1. **Edge gateway (`SGW`).** Shopee's own reverse proxy/WAF — TLS, HSTS, CSP, coarse rate limiting.
   No third-party CDN challenge.
2. **Client-side security SDK (`antifraudivs` / `shopee-trackingsdk` / `shpsec`).** JavaScript that
   fingerprints the device (canvas, WebGL, `navigator.*`, timing), mints `SPC_F` / `security_device_id`,
   and reports to `df.infra.sz.shopee.co.id`. The heavy lifting is lazy-loaded (WASM-style).
3. **Tiered gateway enforcement.** Public reads are open; personalized/abusable endpoints return
   `403 action_type:2 → redirect_to_error_page`; navigation over a risk threshold is redirected to
   `/verify/traffic`.
4. **Risk scoring.** Partly deterministic (endpoint sensitivity), partly a score from SDK signals,
   request velocity and IP reputation. It **soft-fails** — a structured JSON error, not a blank 403.

The practical reverse-engineering insight: you rarely need to beat the browser challenge. The read
APIs are directly callable; only personalized endpoints need the device token, which is minted by
running the real SDK. The data is mostly not behind the wall.

## What Tenet now does with this

Three capabilities, all visible in a scan's findings and endpoints:

- **Endpoint gating.** The browser engine joins each request to its response by request id and
  records the status. An observed API call that returns 401/403/429 becomes a `gated_endpoint`
  finding — turning "here are the endpoints" into "here are the open ones and here are the gated
  ones". On Shopee this cleanly separates `get_category_tree` (open) from
  `recommend/recommend` (403).
- **Anti-bot vendor detection.** `bot-protection` fingerprints for DataDome, Akamai Bot Manager,
  PerimeterX, Kasada and Shopee's own `shpsec` / `antifraudivs` / `trackingsdk`, by script url and
  cookie — plus the existing `detect_challenge`, which records the interstitial when a scan is
  blocked.
- **WASM reverse engineering.** When a rendered page loads a `.wasm`, Tenet fetches it and
  `tenet-wasm` reports its host imports, exports, embedded strings, toolchain and whether it looks
  like a fingerprint/anti-fraud module. Demonstrated end to end against
  `examples/rkm-sec-wasm` — a device-token module built for RKM's own hardening:

  ```
  WebAssembly module      7 functions, 2 imports, 8 exports, host modules ["rkm_host"]
  WASM anti-fraud module  bot-protection — fingerprints the client and mints a token (2 host calls)
  WASM toolchain          rustc 1.96.0   (read from the wasm producers section)
  ```

## The mirror image: hardening RKM

`market.rajawalikaryamulya.co.id` runs Cloudflare + a Vite SPA + Midtrans, on a Cloudflare Workers +
Neon monorepo. It already has Shopee's Layer 1 (Cloudflare) but none of 2–4. The hardening path maps
directly onto the model above:

| Shopee layer | RKM has | Add |
| --- | --- | --- |
| 1 Edge WAF/ratelimit (`SGW`) | Cloudflare | Bot Fight Mode, WAF rulesets, Rate Limiting Rules |
| 2 Client fingerprint + token (`SPC_F`) | — | Cloudflare Turnstile on login/checkout/coupon; a signed device token (the `examples/rkm-sec-wasm` module is the seed) |
| 3 Tiered gating (`action_type:2`) | — | Cache public catalog; require a validated session + rate limit on cart/checkout/price/stock/auth |
| 4 Risk scoring | — | Velocity per IP/token/route in a Durable Object; soft-challenge with Turnstile |

The single most important lesson: **classify endpoints by abuse value and gate accordingly**, and
never trust client-supplied price/stock on the Midtrans checkout path.

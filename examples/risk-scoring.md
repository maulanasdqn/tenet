# How Shopee's server-side risk scoring works (inferred)

Server-side scoring cannot be read directly — the engine runs on Shopee's servers, not in anything
they send you. So this is **black-box inference**: vary one input at a time with single, low-volume
requests and watch the decision (`200` / `403` / challenge). The decision boundary reveals which
signals the engine weighs. This is authorised-testing methodology for understanding a protection so
you can build your own; do it only on systems you own or may assess, and at trivial volume.

## The probe matrix (each row is one request, one variable changed)

| Input | Open endpoint (`get_category_tree`) | Gated endpoint (`recommend` / `pdp/get_pc`) |
| --- | --- | --- |
| Full browser UA + Referer + Accept-Language | **200** | **403** |
| Bot UA (`python-requests/2.31`) | **200** | — |
| No User-Agent / empty UA / no headers | **200** | — |
| No Referer, no Accept-Language | **200** | — |
| `+ sec-ch-ua + sec-fetch-* + x-api-source + x-shopee-language` | — | **403** |
| No cookies | 200 | **403** |
| **Full encrypted session cookies** (`SPC_SEC_SI` + `SPC_T_ID`) | — | **403** |
| Real-Chrome session that ran the SDK (browser engine) | 200 | **called it, not rejected as gated** |

## What the boundary tells us

1. **The first gate is endpoint sensitivity, not the client.** Public catalog reads return `200` to
   *anything* — a bot UA, an empty UA, no headers at all. Shopee does no UA/header bot-blocking on
   low-value data. Only personalized/abusable endpoints (recommend, PDP, flash-sale) enter the risk
   engine.

2. **For gated endpoints the decision is not UA, not headers, not cookies.** A complete,
   browser-perfect header set still returns `403`. Carrying the **full encrypted session cookies**
   — including the HttpOnly `SPC_SEC_SI` and `SPC_T_ID` — still returns `403`. So possession of the
   session is not the same as the session being *trusted*.

3. **The gate is a *verified* session, and verification is server-side state.** The client holds an
   opaque encrypted cookie; whether that session is "verified" lives in Shopee's session store, keyed
   to the cookie. You cannot forge "verified" because it is not in the cookie — it is on the server.
   A session becomes verified only when a **real browser runs the security SDK, produces a device
   fingerprint, and clears the risk check**. That is exactly why `curl` with perfect cookies fails
   while the browser engine's real-Chrome session reached `pdp/get_pc` without being rejected: `curl`
   can't execute the JS to earn a verified session; Chrome can.

## The inferred scoring inputs

The score itself is invisible, but the boundary — plus the client SDK's telemetry to
`df.infra.sz.shopee.co.id/v2/shpsec/web/report` and the datacenter-IP challenge on the detail page —
points to these inputs, in rough order of weight:

1. **Verified-session state** — did this session pass the JS challenge? (the hard gate)
2. **IP reputation** — datacenter/hosting ranges score high-risk; a residential IP is the biggest
   single lever a real user has and a bot lacks. (The detail-page challenge fired from a datacenter
   IP even with clean client signals.)
3. **Device fingerprint** — the SDK's canvas/WebGL/navigator/timing hash, and whether it is stable
   and human-plausible.
4. **Request velocity / rate** — per IP, per fingerprint, per session, per endpoint. (Silent — no
   `ratelimit-*` headers; enforced server-side.)
5. **TLS/JA3 + HTTP/2 fingerprint** — does the handshake match the claimed browser?
6. **Login state** — `is_login:false` appears in every gated `403`; being logged in is a strong
   trust signal.
7. **Endpoint sensitivity** — a multiplier: the same session may pass on the homepage but be
   challenged on a product page.

Output is **soft-fail**: over threshold ⇒ `403 {action_type:2, redirect_to_error_page}` ⇒
`/verify/traffic`, so a real user gets a path instead of a blank wall.

## The one mechanism that makes it unbreakable from the client

There is no client secret to steal. The lock is **server-side session state (verified + risk score)
keyed to an opaque cookie.** Everything the client holds is either opaque (encrypted cookie) or a
mere signal (fingerprint) that the server *re-evaluates*. Perfect client emulation only improves your
score; it never sets `verified=true`, because that bit lives on Shopee's server.

## Building the same thing for RKM

This is directly copyable on Cloudflare Workers + Neon:

1. **Keep verified-session state on the server.** Issue an opaque, HttpOnly, encrypted session cookie
   from a Worker. Store `{verified: bool, risk: int, fingerprint, ip, issued_at}` in a Durable Object
   or Neon, keyed to the cookie. The client never sees or controls `verified`.
2. **Flip `verified=true` only after a challenge passes** — Cloudflare Turnstile plus your own signal
   checks (JA3 via Cloudflare, header coherence, device fingerprint).
3. **Gate sensitive endpoints on server-side `verified` + a fresh risk score**, never on cookie
   possession. Public catalog stays open and cached.
4. **Score the inputs you can see at the edge**: Cloudflare gives you JA3, IP reputation, ASN
   (datacenter vs residential), and rate. Add velocity counters in a Durable Object.
5. **Soft-fail to a Turnstile challenge**, not a blank 403, so real users pass.

The lesson the whole investigation converges on: Shopee is hard to beat not because of clever client
code — it ships none — but because **the decision is server-side state you cannot forge.** Put your
budget there.

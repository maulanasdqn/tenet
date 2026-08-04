# Why a real browser works but an automated tool gets blocked

The site content is identical. What differs is the *signals* the two produce, and a risk engine like
Shopee's `SGW` scores those signals across six layers. A human in a real browser looks consistent at
every layer; an automated tool leaks a tell at one or more. This is authorised-testing material —
use it to understand detection and harden your own site, on targets you own or may assess.

## The six layers, bottom to top

| # | Layer | What a real browser produces | What automation leaks |
| --- | --- | --- | --- |
| 1 | **TLS fingerprint (JA3/JA4)** | Chrome's exact ClientHello — cipher order, extensions, GREASE, ALPN | `curl`/`requests`/Go send a different ClientHello. **A perfect browser User-Agent over curl still fails here** — the handshake says curl |
| 2 | **HTTP/2 fingerprint** | Chrome's SETTINGS frame order and pseudo-header order | Libraries send different frame/header order |
| 3 | **Header consistency** | UA matches `sec-ch-ua`, `Accept-Language`, `sec-fetch-*` all present and coherent | Missing/mismatched client hints and fetch metadata |
| 4 | **JS fingerprint** | `navigator.webdriver=false`, real plugins, a real GPU in WebGL, coherent screen/timezone | `webdriver=true`, no plugins, WebGL renderer `SwiftShader`/`llvmpipe` (headless GPU), `HeadlessChrome` in UA |
| 5 | **Behaviour** | Mouse movement, scroll, focus/blur, timing entropy from a human | Navigates instantly, **zero mouse or scroll events** |
| 6 | **Session-establishment flow** | Loads HTML → runs the SDK → mints the fingerprint → gets the encrypted session → *then* calls the API | Jumps straight to the API with no valid `SPC_SEC_SI` session |

The single biggest reason a "browser-UA automated tool" is blocked while a real browser is not:
**layers 1 and 6** — the TLS handshake betrays the client before any JS runs, and skipping the JS
flow means there is no valid, server-issued encrypted session. For a *headless browser* specifically,
the tells are **layers 4 and 5** — the headless GPU/`webdriver` markers and the total absence of
human interaction.

## What Tenet does about each layer

| Layer | Tenet's answer | Where |
| --- | --- | --- |
| 1 TLS | The **browser engine uses real Chrome**, so its TLS/JA3 is genuine. The `http` (reqwest) engine does **not** match Chrome — a known gap; a TLS-impersonating client is the fix | inherent to `engine=browser` |
| 2 HTTP/2 | Same — real Chrome for `engine=browser` | inherent |
| 3 Headers | UA + client hints + `Accept-Language` overridden coherently by `BROWSER_REGION` | `tenet-stealth` |
| 4 JS fingerprint | `navigator.webdriver=false`, plugins, vendor, a realistic WebGL renderer, languages | `tenet-stealth` init script |
| 5 Behaviour | **A human-like interaction pass** — mouse moves, scrolls and dwells through Chrome's real input pipeline | `tenet-stealth::interaction_plan` + `tenet-browser::humanize` |
| 6 Session flow | The engine navigates the page and lets its scripts run before reading, so a real session is established | `tenet-browser` render flow |

### The behaviour layer — measured

Against a fixture whose own listeners count `mousemove`/`wheel`/`scroll` events:

```
BROWSER_BEHAVIOR=0   →  moves_0  / wheels_0     (the classic bot signature)
BROWSER_BEHAVIOR=1   →  moves_19 / wheels_12    (real events via Chrome's input pipeline)
```

The plan is deterministic per url (seeded), stays inside the viewport, and interleaves dwells, so
it reads as a person glancing around a page rather than a script. `BROWSER_BEHAVIOR=1` is the
default; set it to `0` to reproduce the bot signature.

## The honest limits

Stealth normalises what it can reach; it is **not** a bypass:

- **The `http` engine's TLS/JA3 is still curl-shaped.** Only `engine=browser` has a genuine Chrome
  fingerprint. Fixing the static engine needs a TLS-impersonating HTTP client (the `curl-impersonate`
  / Chrome-JA3 approach).
- **IP reputation is unaddressed.** A datacenter or flagged IP is a strong signal on its own; a
  residential egress is a separate concern Tenet does not manage.
- **The server-side risk engine can still win.** Shopee's decision lives in `SGW`, behind an
  encrypted session you cannot forge. Perfect client signals raise your score; they do not guarantee
  a pass.

## Turning this around for RKM

Every row above is also a *detection* you can deploy. To tell a bot from a human on your own site:

1. **JA3/JA4 at the edge** — Cloudflare exposes the TLS fingerprint; flag clients whose TLS does not
   match their claimed browser.
2. **Header coherence** — reject requests where UA, client hints and `Accept-Language` disagree.
3. **A JS challenge** (Turnstile) that checks `navigator.webdriver`, the WebGL renderer and the rest.
4. **Behavioural telemetry** — a session with zero mouse/scroll before a sensitive action is a bot;
   `moves_0/wheels_0` is exactly the signal to score on.
5. **Require the full session flow** — issue an encrypted, HttpOnly session token only after the
   client passes the challenge, and gate sensitive endpoints on it. This is the Shopee `SPC_SEC_SI`
   move, and it is the one that actually holds.

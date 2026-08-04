# rkm-sec (WebAssembly device-token module)

A minimal client-side device-token module, modeled on the pattern Shopee uses (`antifraudivs`):
JavaScript collects raw signals, WebAssembly hides the token algorithm.

This is a starting point for hardening RKM, and the fixture `tenet-wasm` reverse-engineers.

## Build

```bash
cargo build --release --target wasm32-unknown-unknown
# -> target/wasm32-unknown-unknown/release/rkm_sec.wasm
```

It is excluded from the Tenet workspace (wasm-only cdylib). Copy the built `.wasm` into
`tenet-wasm/tests/fixtures/rkm_sec.wasm` to refresh the reverse-engineering fixture.

## Shape

- Imports (`rkm_host`): `host_now_ms`, `host_entropy` — supplied by the page's JS glue.
- Exports: `rkm_alloc`, `rkm_version`, `rkm_fingerprint(ptr,len)`, `rkm_device_token(fp)`, `rkm_marker`.
- The token algorithm (keyed hash + rotation + host entropy) lives in the binary, not in readable JS.

## How a page would use it

```js
const imports = { rkm_host: { host_now_ms: () => Date.now(), host_entropy: () => crypto.getRandomValues(new Uint32Array(1))[0] } };
const { instance } = await WebAssembly.instantiateStreaming(fetch('/rkm_sec.wasm'), imports);
// collect signals in JS, hash them through rkm_fingerprint, mint the token with rkm_device_token,
// send it as a header on sensitive API calls and validate it in a Cloudflare Worker.
```

To make this production hardening you still need the server side: a Worker that issues a rotating
key, validates the token, and rate-limits by it — the WASM only hides the client algorithm.

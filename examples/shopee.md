# Sample scan: shopee.co.id

A real run of both engines against a production single-page app, kept here because it shows the
difference between reading a page and watching one. Reproduce it with:

```bash
scripts/sample-scan.sh https://shopee.co.id http
scripts/sample-scan.sh https://shopee.co.id browser
```

Everything below comes from content the site serves to any browser, read once at ordinary visit
volume. Scan targets you own or are authorised to assess.

## The headline

| | `engine: http` | `engine: browser` |
| --- | --- | --- |
| Endpoints found | **1** | **58** |
| Observed at runtime | 0 | 53 |
| Technology detected | 0 | Google Tag Manager, Hotjar |
| Auth schemes | none | `bearer` (0.95), `api_key`, token storage |

Shopee serves a shell page and builds everything else in the browser, so static reading has almost
nothing to work with. The one endpoint the http engine found — `/api/v4/pages/is_short_url` — came
from a bare string literal at confidence 0.45.

## What the browser engine saw

Its own API, with the verbs the app really used:

```
POST   https://shopee.co.id/api/v4/abtest/traffic/get_web_experiments
GET    https://shopee.co.id/api/v4/account/basic/get_account_info
GET    https://shopee.co.id/api/v4/account/basic/get_payment_info
GET    https://shopee.co.id/api/v4/account/subaccount
POST   https://shopee.co.id/api/v4/banner/batch_list
POST   https://shopee.co.id/api/v4/banner/batch_list_by_spaces
GET    https://shopee.co.id/api/v4/flash_sale/flash_sale_get_items
GET    https://shopee.co.id/api/v4/homepage/campaign_modules
GET    https://shopee.co.id/api/v4/pages/get_category_tree
GET    https://shopee.co.id/api/v4/pages/get_footer_layout
GET    https://shopee.co.id/api/v4/pages/get_homepage_category_list
GET    https://shopee.co.id/api/v4/platform/get_ft_v2
GET    https://shopee.co.id/api/v4/recommend/recommend
GET    https://shopee.co.id/api/v4/search/search_prefills
GET    https://shopee.co.id/api/v4/search/search_suggestion
POST   https://shopee.co.id/api/v4/web/subcart
```

Every one of these is confidence 0.95 with source `runtime` — the request was observed leaving the
browser, not guessed from a bundle.

Three results worth calling out:

- `POST https://dem.shopee.com/dem/janus/v1/app-auth/login` — an auth exchange on a separate
  service, flagged as an `auth_endpoint`, with `bearer` pinned at 0.95 from an `Authorization`
  header seen on the wire.
- `POST https://df.infra.sz.shopee.co.id/v2/shpsec/web/report` — the client-side anti-fraud
  reporting channel, which never appears in the served HTML.
- `GET https://qafeautomation.sra.test.shopee.io/api/executions/batch` — an internal QA
  automation hostname referenced from a public bundle. Found statically, at 0.45.

## Bot protection is visible in the results

An earlier attempt was redirected to `https://shopee.co.id/verify/traffic/error?home_url=...`, the
traffic-verification interstitial. The scan still succeeded and still reported the trackers that
loaded on the challenge page — it simply found fewer of the real endpoints. If a browser scan comes
back thin, check the rendered artifact's url before assuming the site has no API.

## Where the noise is

Of 58 endpoints, 24 sit on `deo.shopeemobile.com` and are CDN content payloads
(`.../id.col106.1789...`) that the app happens to fetch with `fetch()`. They are genuinely observed
requests, so Tenet records them, but they crowd out the 21 API-shaped paths. Reading the endpoint
list grouped by origin is the practical workaround:

```
  24  https://deo.shopeemobile.com          content delivery
  17  https://shopee.co.id                  the actual api
   3  https://dem.shopee.com                telemetry and app auth
   3  https://ad.doubleclick.net            advertising
   3  https://www.google.com                advertising
   1  https://df.infra.sz.shopee.co.id      anti-fraud
   1  https://qafeautomation.sra.test...    internal qa (static reference)
```

Distinguishing an API call from a CDN payload is an open question — see the roadmap in the README.

## The generated spec

`GET /v1/scans/{id}/openapi` returns an OpenAPI 3.1 document with `https://shopee.co.id` as the
server and per-operation `servers` overrides for the third-party origins, so the mixed-origin
reality is preserved rather than flattened:

```json
{
  "openapi": "3.1.0",
  "info": { "title": "https://shopee.co.id/", "version": "0.1.0" },
  "servers": [{ "url": "https://shopee.co.id" }],
  "paths": {
    "/api/v4/banner/batch_list": {
      "post": {
        "operationId": "post_api_v4_banner_batch_list",
        "summary": "POST /api/v4/banner/batch_list",
        "responses": { "default": { "description": "observed during reverse engineering" } },
        "x-tenet-confidence": 0.95,
        "x-tenet-source": "runtime"
      }
    }
  }
}
```

Request and response bodies are not captured yet, so the document describes the surface rather than
the contract.

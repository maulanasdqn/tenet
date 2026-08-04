#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Header(&'static str),
    Cookie,
    Body,
    ScriptUrl,
}

impl Source {
    pub fn label(&self) -> &'static str {
        match self {
            Source::Header(name) => name,
            Source::Cookie => "set-cookie",
            Source::Body => "body",
            Source::ScriptUrl => "script-url",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Signature {
    pub name: &'static str,
    pub category: &'static str,
    pub source: Source,
    pub needle: &'static str,
    pub confidence: f32,
}

const fn head(
    header: &'static str,
    needle: &'static str,
    name: &'static str,
    category: &'static str,
    confidence: f32,
) -> Signature {
    Signature {
        name,
        category,
        source: Source::Header(header),
        needle,
        confidence,
    }
}

const fn cookie(
    needle: &'static str,
    name: &'static str,
    category: &'static str,
    confidence: f32,
) -> Signature {
    Signature {
        name,
        category,
        source: Source::Cookie,
        needle,
        confidence,
    }
}

const fn body(
    needle: &'static str,
    name: &'static str,
    category: &'static str,
    confidence: f32,
) -> Signature {
    Signature {
        name,
        category,
        source: Source::Body,
        needle,
        confidence,
    }
}

const fn script(
    needle: &'static str,
    name: &'static str,
    category: &'static str,
    confidence: f32,
) -> Signature {
    Signature {
        name,
        category,
        source: Source::ScriptUrl,
        needle,
        confidence,
    }
}

pub const SIGNATURES: &[Signature] = &[
    head("server", "cloudflare", "Cloudflare", "cdn", 0.95),
    head("cf-ray", "", "Cloudflare", "cdn", 0.9),
    head("x-vercel-id", "", "Vercel", "hosting", 0.95),
    head("x-amz-cf-id", "", "Amazon CloudFront", "cdn", 0.9),
    head("x-fastly-request-id", "", "Fastly", "cdn", 0.95),
    head("x-akamai-transformed", "", "Akamai", "cdn", 0.9),
    head("x-nf-request-id", "", "Netlify", "hosting", 0.95),
    head("x-github-request-id", "", "GitHub Pages", "hosting", 0.8),
    head("server", "nginx", "nginx", "server", 0.9),
    head("server", "apache", "Apache", "server", 0.9),
    head("server", "caddy", "Caddy", "server", 0.9),
    head("server", "envoy", "Envoy", "proxy", 0.85),
    head("server", "gunicorn", "Gunicorn", "server", 0.9),
    head("server", "kestrel", "ASP.NET Kestrel", "server", 0.9),
    head("x-powered-by", "express", "Express", "backend", 0.9),
    head("x-powered-by", "php", "PHP", "language", 0.9),
    head("x-powered-by", "next.js", "Next.js", "frontend", 0.95),
    head("x-powered-by", "asp.net", "ASP.NET", "backend", 0.9),
    head("x-runtime", "", "Ruby on Rails", "backend", 0.6),
    head("x-drupal-cache", "", "Drupal", "cms", 0.9),
    head("x-shopid", "", "Shopify", "ecommerce", 0.95),
    head("x-shopify-stage", "", "Shopify", "ecommerce", 0.95),
    head("x-amzn-waf-action", "", "AWS WAF", "waf", 0.9),
    head("x-iinfo", "", "Imperva", "waf", 0.9),
    head("x-sucuri-id", "", "Sucuri", "waf", 0.9),
    cookie("laravel_session", "Laravel", "backend", 0.9),
    cookie("csrftoken", "Django", "backend", 0.8),
    cookie("phpsessid", "PHP", "language", 0.9),
    cookie("jsessionid", "Java", "language", 0.9),
    cookie("asp.net_sessionid", "ASP.NET", "backend", 0.9),
    cookie("connect.sid", "Express", "backend", 0.85),
    cookie("bigipserver", "F5 BIG-IP", "waf", 0.9),
    cookie("__cf_bm", "Cloudflare Bot Management", "waf", 0.9),
    cookie("incap_ses", "Imperva Incapsula", "waf", 0.9),
    cookie("wordpress_logged_in", "WordPress", "cms", 0.95),
    body("/_next/static", "Next.js", "frontend", 0.95),
    body("__next_data__", "Next.js", "frontend", 0.95),
    body("__nuxt__", "Nuxt", "frontend", 0.95),
    body("data-reactroot", "React", "frontend", 0.8),
    body("ng-version", "Angular", "frontend", 0.9),
    body("__svelte", "Svelte", "frontend", 0.8),
    body("__remixcontext", "Remix", "frontend", 0.9),
    body("astro-island", "Astro", "frontend", 0.9),
    body("wp-content", "WordPress", "cms", 0.9),
    body("/graphql", "GraphQL", "api", 0.7),
    script("js.stripe.com", "Stripe", "payments", 0.95),
    script(
        "googletagmanager.com",
        "Google Tag Manager",
        "analytics",
        0.95,
    ),
    script(
        "google-analytics.com",
        "Google Analytics",
        "analytics",
        0.95,
    ),
    script("cdn.segment.com", "Segment", "analytics", 0.95),
    script("browser.sentry-cdn.com", "Sentry", "monitoring", 0.95),
    script("datadoghq-browser-agent", "Datadog RUM", "monitoring", 0.9),
    script("recaptcha", "reCAPTCHA", "captcha", 0.9),
    script(
        "challenges.cloudflare.com",
        "Cloudflare Turnstile",
        "captcha",
        0.95,
    ),
    script("hcaptcha.com", "hCaptcha", "captcha", 0.95),
    script("supabase", "Supabase", "backend", 0.8),
    script("firebaseapp.com", "Firebase", "backend", 0.9),
    script("auth0.com", "Auth0", "auth", 0.9),
    script("clerk.", "Clerk", "auth", 0.85),
    script("algolia", "Algolia", "search", 0.85),
    script("intercom", "Intercom", "support", 0.9),
    script("static.hotjar.com", "Hotjar", "analytics", 0.9),
    script("datadome", "DataDome", "bot-protection", 0.9),
    script("captcha.px-cdn.net", "PerimeterX", "bot-protection", 0.9),
    script("client.px-cloud.net", "PerimeterX", "bot-protection", 0.9),
    script("kasada", "Kasada", "bot-protection", 0.85),
    script(
        "shopee-trackingsdk",
        "Shopee tracking SDK",
        "bot-protection",
        0.9,
    ),
    script(
        "antifraudivs",
        "Shopee Anti-Fraud IVS",
        "bot-protection",
        0.9,
    ),
    script("shpsec", "Shopee shpsec", "bot-protection", 0.9),
    cookie("datadome", "DataDome", "bot-protection", 0.9),
    cookie("_abck", "Akamai Bot Manager", "bot-protection", 0.9),
    cookie("bm_sz", "Akamai Bot Manager", "bot-protection", 0.85),
    cookie("_pxvid", "PerimeterX", "bot-protection", 0.9),
    cookie("px_cookie", "PerimeterX", "bot-protection", 0.85),
    cookie("__kpsdk", "Kasada", "bot-protection", 0.85),
    cookie("spc_f", "Shopee device token", "bot-protection", 0.85),
    cookie(
        "security_device_id",
        "Shopee trusted-device token",
        "bot-protection",
        0.85,
    ),
];

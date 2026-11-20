# [<img src="https://s3.internetdata.io/internetdata-public/brand/mark.svg" alt="InternetData" height="28"/>](https://internetdata.io/) InternetData axum Middleware

[![crates.io](https://img.shields.io/crates/v/internetdata-axum.svg)](https://crates.io/crates/internetdata-axum)
[![docs.rs](https://img.shields.io/docsrs/internetdata-axum)](https://docs.rs/internetdata-axum)
[![license](https://img.shields.io/github/license/internetdata/sdk-rust-axum.svg)](LICENSE)

The official [axum](https://github.com/tokio-rs/axum) middleware for the [InternetData](https://internetdata.io) API.

It looks up the visitor behind each request, where they are, which network they come from and whether it's a VPN, a proxy, Tor or a hosting server, and puts the answer on the request. Blocking is opt-in.

It is a `tower::Layer`, so it works in any tower stack, not only axum.

## Getting Started

```bash
cargo add internetdata-axum internetdata
```

Requires Rust 1.85 or newer, and axum 0.8.

Without an API key a lookup is on Unlimited, the free plan: the country, the network and the company behind the address, Tor, relay, anycast, carrier and satellite space and the service flags, with no daily or monthly cap. Every lookup your app makes comes from your server's one address, though, so a busy site will see bursts refused. A paid plan answers every field, VPN, the proxy flags, hosting, CDN and the city among them; create a key in the [console](https://app.internetdata.io). A free account tries every field.

```rust
use axum::{Router, routing::get};
use internetdata::middleware::Options;
use internetdata_axum::{Found, InternetData};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let guard = InternetData::new(
        Options::new().api_key(std::env::var("INTERNETDATA_API_KEY").unwrap_or_default()),
    )?;

    let app = Router::new().route("/", get(hello)).layer(guard);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await?;
    Ok(())
}

async fn hello(Found(found): Found) -> String {
    match &found.result {
        Some(result) => format!("Hello, visitor from {}", result.location.country_name),
        None => "Hello".to_owned(),
    }
}
```

By default nothing is blocked. Every request carries a `Lookup` and your own handlers decide what it means: a currency to show, a sign-up to review, a page to keep from some countries.

`Found` rejects with `500` on a route the layer does not cover, which is a wiring mistake rather than something a visitor can cause. Use `MaybeFound` where that is deliberate.

## Blocking

Set a `block_condition` and a matching request is answered with `403` and never reaches your handlers.

```rust
use internetdata::middleware::{Condition, Options};

Options::new().block_condition(vec![Condition::from([(
    "location",
    Condition::from([("country", vec!["CU", "IR", "KP"])]),
)])]);
```

A condition is written in the shape of a result, keyed by the same names the API uses, and only the members you name are considered, so it can refuse on anything in the answer. The country works on every plan, and so do the first two of these:

```rust
use internetdata::middleware::{Bound, Condition};

// the network's type
Condition::from([("asn", Condition::from([("type", "hosting")]))]);

// anycast and satellite space: a list is OR
vec![Condition::from([("is_anycast", true)]), Condition::from([("is_satellite", true)])];

// traffic from servers, not people
vec![Condition::from([("is_hosting", true)]), Condition::from([("is_cdn", true)])];

// anonymity
vec![
    Condition::from([("is_vpn", true)]),
    Condition::from([("is_tor", true)]),
    Condition::from([("is_resproxy", true)]),
];

// one provider
Condition::new()
    .with("is_vpn", true)
    .with("vpn", Condition::from([("provider", "nordvpn")]));

// a numeric threshold
Condition::from([("resproxy", Condition::from([("hits", Bound::gte(5.0))]))]);
```

Values are matched by equality, strings without regard to case. A `Vec` means any-of. `Bound::gte`, `gt`, `lte` and `lt` compare numbers and chain into a range (`Bound::gte(5.0).and_lt(100.0)`); every bound you give must hold. Members set to `false` or `Value::Null` are ignored, so a condition states the signals you act on; one that constrains nothing would match every request, and is refused when the middleware is built rather than silently blocking all your traffic.

A policy that lives in your app's config rather than in its source is read with `from_json`, which takes the same shape:

```rust
let condition = internetdata::middleware::from_json(&serde_json::json!({
    "location": { "country": ["CU", "IR", "KP"] },
}))?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

Replace the refusal with `on_blocked`:

```rust
# use axum::response::IntoResponse;
# use internetdata::middleware::Options;
# fn f() -> Result<(), internetdata::Error> {
internetdata_axum::InternetData::new(Options::new())?
    .on_blocked(|_parts, _found| {
        (axum::http::StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS, "Not available in your country")
            .into_response()
    });
# Ok(())
# }
```

## Where the client address comes from

This is the setting that decides whether any of the above works, and it is the one thing only you can get right.

By default the middleware reads axum's `ConnectInfo<SocketAddr>`, which is the socket peer, and which is **absent entirely unless you serve the app with `into_make_service_with_connect_info`**, as the example above does. Behind nginx, a load balancer, or a CDN, that peer is your proxy, and that is the address that gets looked up: every visitor gets your proxy's country and network, and if it is a hosting address, a hosting rule blocks everyone. nginx on the same machine is `127.0.0.1` and a load balancer usually has a private address, which the middleware answers locally without a lookup, as an address with no country, and logs a warning.

For an edge that writes the address into its own header, name the header:

```rust
use internetdata::middleware::{Options, header};

Options::new().ip_selector(header("CF-Connecting-IP")); // or True-Client-IP
```

`xff(0)` reads the left-most `X-Forwarded-For` entry. Be aware that the left-most entry is whatever the caller sent, because proxies append to that header, so it's only trustworthy when an edge you control overwrites it. If you know how many proxies sit in front, count from the right instead: `xff(1)` is the address your nearest proxy saw.

Anything else, pass your own closure. It receives a `RequestView` and returns an address:

```rust
use std::sync::Arc;
use internetdata::middleware::{IpSelector, Options};

let mine: IpSelector = Arc::new(|view| (view.header)("x-real-ip"));
Options::new().ip_selector(mine);
```

If the address resolves to a private one, the middleware says so once through `on_warn`. That is expected on localhost and is the signal to fix your configuration anywhere else.

## When a lookup fails

The request is let through, and the reason is on `Lookup::error`. Our outage should not become yours, so a network failure, a rate limit or a rejected key all fail open.

```rust
# use internetdata_axum::Found;
async fn handler(Found(found): Found) -> &'static str {
    if let Some(error) = &found.error {
        tracing::warn!(%error, "internetdata unavailable");
    }
    "ok"
}
```

Set `fail_closed(true)` to block instead. Private addresses are answered locally and never fail, so this will not lock you out in development.

## Cost and latency

Answers are cached per middleware for an hour, so a returning visitor costs nothing, and private addresses never leave the process. A cache miss is one request to our API, bounded at 2500 ms by default and not retried: on a request path, failing open quickly beats holding a visitor while we try again. Both are adjustable, and so is the cache, through a client you build yourself and pass as `client`.

Layer the routes that matter rather than the whole router, or skip what you do not care about:

```rust
# use internetdata::middleware::Options;
# fn f() -> Result<(), internetdata::Error> {
internetdata_axum::InternetData::new(Options::new())?
    .skip(|parts| parts.uri.path().starts_with("/healthz"));
# Ok(())
# }
```

If you already hold an `internetdata::Client`, pass it as `client` and the middleware will share it rather than building a second cache.

Beyond a few million distinct visitors a day, stop calling the API per request: look addresses up locally in [our databases](https://internetdata.io/databases) instead.

## Absent is not false

Unlimited, with a key or without one, leaves out `is_vpn`, `is_hosting`, `is_cdn` and the four proxy flags, the detail behind them, and the paid members inside `location`, `asn`, `company` and `abuse`, such as `location.city`. A field your plan does not include is `None`, which means "not in your plan" rather than "checked, and no".

```rust
# fn f(result: &internetdata::Lookup) -> bool {
result.is_vpn.unwrap_or(false)   // when you only want the flag
# }
```

A `block_condition` naming a member your plan does not serve, `location.city` on Unlimited included, can never match, so the middleware warns once instead of failing silently. Set `on_missing_field(OnMissingField::Fail)` to make it an error.

## Other Libraries

There are official InternetData client libraries available for many languages including PHP, Python, Go, Java, and Ruby. See our GitHub at https://github.com/internetdata for more.

## About InternetData

Geolocation, anonymity, ownership and network data for IP addresses, AS numbers and domains, as an API and as databases you download and query yourself.

[<img src="https://s3.internetdata.io/internetdata-public/brand/mark.svg" alt="InternetData" height="64"/>](https://internetdata.io/)

## License

This project is licensed under the [MIT License](LICENSE).

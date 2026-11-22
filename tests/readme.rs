// Every code block in README.md, compiled but never run.
//
// The README is the API contract customers actually read, so a rename or a
// signature change that invalidates it should fail the build rather than reach
// a reader. Mirror any README edit here.
#![allow(unused, path_statements, clippy::no_effect)]

use std::sync::Arc;

use axum::response::IntoResponse;
use axum::{Router, routing::get};
use internetdata::middleware::{
    Bound, Condition, IpSelector, OnMissingField, Options, header, xff,
};
use internetdata_axum::{Found, InternetData, MaybeFound};

async fn getting_started() -> Result<(), Box<dyn std::error::Error>> {
    let guard = InternetData::new(
        Options::new().api_key(std::env::var("INTERNETDATA_API_KEY").unwrap_or_default()),
    )?;

    let app = Router::new().route("/", get(hello)).layer(guard);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app.into_make_service_with_connect_info::<std::net::SocketAddr>())
        .await?;
    Ok(())
}

async fn hello(Found(found): Found) -> String {
    match &found.result {
        Some(result) => format!("Hello, visitor from {}", result.location.country_name),
        None => "Hello".to_owned(),
    }
}

fn blocking() -> Result<(), Box<dyn std::error::Error>> {
    Options::new().block_condition(vec![Condition::from([(
        "location",
        Condition::from([("country", vec!["CU", "IR", "KP"])]),
    )])]);

    // the network's type
    Condition::from([("asn", Condition::from([("type", "hosting")]))]);

    // anycast and satellite space: a list is OR
    vec![Condition::from([("is_anycast", true)]), Condition::from([("is_satellite", true)])];

    // traffic from servers, not people
    vec![Condition::from([("is_hosting", true)]), Condition::from([("is_cdn", true)])];

    // crawlers and other bots, on every plan
    Condition::from([("is_bot", true)]);

    // anonymity
    vec![
        Condition::from([("is_vpn", true)]),
        Condition::from([("is_tor", true)]),
        Condition::from([("is_resproxy", true)]),
    ];

    // one provider
    Condition::new().with("is_vpn", true).with("vpn", Condition::from([("provider", "nordvpn")]));

    // a numeric threshold
    Condition::from([("resproxy", Condition::from([("hits", Bound::gte(5.0))]))]);

    Bound::gte(5.0).and_lt(100.0);

    let condition = internetdata::middleware::from_json(&serde_json::json!({
        "location": { "country": ["CU", "IR", "KP"] },
    }))?;

    InternetData::new(Options::new())?.on_blocked(|_parts, _found| {
        (axum::http::StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS, "Not available in your country")
            .into_response()
    });
    Ok(())
}

fn selectors() -> Result<(), internetdata::Error> {
    Options::new().ip_selector(header("CF-Connecting-IP")); // or True-Client-IP
    Options::new().ip_selector(xff(0));
    Options::new().ip_selector(xff(1));

    let mine: IpSelector = Arc::new(|view| (view.header)("x-real-ip"));
    Options::new().ip_selector(mine);

    Options::new().fail_closed(true);
    Options::new().on_missing_field(OnMissingField::Fail);

    InternetData::new(Options::new())?.skip(|parts| parts.uri.path().starts_with("/healthz"));
    Ok(())
}

async fn failure(Found(found): Found) -> &'static str {
    if let Some(error) = &found.error {
        // tracing::warn!(%error, "internetdata unavailable") in the README; the
        // adapter does not depend on tracing, so this stands in for it.
        eprintln!("internetdata unavailable: {error}");
    }
    "ok"
}

fn absent_is_not_false(result: &internetdata::Lookup) -> bool {
    result.is_vpn.unwrap_or(false)
}

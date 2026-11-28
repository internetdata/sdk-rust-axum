// `allow_bots` through a real axum router, built as the README builds it: a
// crawler's own User-Agent reaches the API, at most four an address, and a
// verified crawler passes the condition it otherwise meets. Every peer is a
// PUBLIC address, since a bogon is answered locally and would pass all of this
// vacuously.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use axum::Router;
use axum::body::Body;
use axum::extract::{ConnectInfo, Path, Query, State};
use axum::routing::get;
use http::{Request, StatusCode};
use internetdata::middleware::{Condition, Options};
use internetdata_axum::{InternetData, MaybeFound};
use serde_json::{Value, json};
use tower::ServiceExt;

const PEER: &str = "66.249.66.1";
const GOOGLEBOT: &str = "Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)";

fn crawler(n: u32) -> String {
    format!("Mozilla/5.0 (compatible; Examplebot/{n}.0)")
}

/// The `user_agent` and the key each lookup carried.
type Seen = Arc<Mutex<Vec<(Option<String>, Option<String>)>>>;

/// A stub API answering every lookup with one verdict, recording what each
/// lookup carried.
#[derive(Clone)]
struct Api {
    verdict: Option<&'static str>,
    /// Answer the verdict only to a lookup that carried an agent, as the API does.
    only_with_agent: bool,
    seen: Seen,
}

impl Api {
    async fn start(verdict: Option<&'static str>) -> (String, Self) {
        Self::serve(verdict, false).await
    }

    async fn start_verifying_agents() -> (String, Self) {
        Self::serve(Some("verified"), true).await
    }

    async fn serve(verdict: Option<&'static str>, only_with_agent: bool) -> (String, Self) {
        let api = Self { verdict, only_with_agent, seen: Arc::new(Mutex::new(Vec::new())) };
        let app = Router::new().route("/{ip}", get(lookup)).with_state(api.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("addr");
        tokio::spawn(async move { axum::serve(listener, app).await.expect("serve") });
        (format!("http://{addr}"), api)
    }

    fn agents(&self) -> Vec<Option<String>> {
        self.seen.lock().unwrap().iter().map(|(agent, _)| agent.clone()).collect()
    }
}

async fn lookup(
    State(api): State<Api>,
    Path(ip): Path<String>,
    Query(query): Query<HashMap<String, String>>,
    headers: http::HeaderMap,
) -> axum::Json<Value> {
    let key = headers.get(http::header::AUTHORIZATION).and_then(|v| v.to_str().ok());
    api.seen.lock().unwrap().push((query.get("user_agent").cloned(), key.map(str::to_owned)));
    let mut body = unlimited();
    body["ip"] = json!(ip);
    body["is_hosting"] = json!(true);
    if let Some(verdict) = api.verdict {
        if !api.only_with_agent || query.contains_key("user_agent") {
            body["user_agent"] = json!({ "agent": "Googlebot", "kind": "bot", "verdict": verdict });
        }
    }
    axum::Json(body)
}

/// The corpus's Unlimited answer: a served answer carries every member the spec
/// requires, so a test's own members go over it.
fn unlimited() -> Value {
    let corpus: Value =
        serde_json::from_str(include_str!("../testdata/testdata.json")).expect("the corpus");
    corpus["lookup"]
        .as_array()
        .expect("lookup cases")
        .iter()
        .find(|case| case["name"] == "unlimited-answer")
        .expect("the corpus has an unlimited-answer lookup case")["body"]
        .clone()
}

/// The README's build, pointed at the stub: a key, `allow_bots` as given, and
/// a condition refusing a hosting address.
fn app(base_url: &str, allow_bots: bool) -> Router {
    let options = Options::new()
        .api_key("key")
        .allow_bots(allow_bots)
        .block_condition(vec![Condition::from([("is_hosting", true)])])
        .base_url(base_url);
    Router::new()
        .route(
            "/",
            get(|MaybeFound(found): MaybeFound| async move {
                let result = found.as_ref().and_then(|f| f.result.as_ref());
                let verdict =
                    result.and_then(|r| r.user_agent.as_ref()).map(|ua| ua.verdict.to_string());
                axum::Json(json!({ "attached": result.is_some(), "verdict": verdict }))
            }),
        )
        .layer(InternetData::new(options).expect("build"))
}

async fn call(app: Router, user_agent: Option<&str>) -> (StatusCode, Value) {
    call_from(app, PEER, user_agent).await
}

async fn call_from(app: Router, peer: &str, user_agent: Option<&str>) -> (StatusCode, Value) {
    let mut request = Request::builder().uri("/");
    if let Some(agent) = user_agent {
        request = request.header(http::header::USER_AGENT, agent);
    }
    let mut request = request.body(Body::empty()).expect("request");
    // The request is built by hand, so no client put a User-Agent of its own on it.
    assert_eq!(request.headers().contains_key(http::header::USER_AGENT), user_agent.is_some());
    let addr: SocketAddr = format!("{peer}:54321").parse().expect("peer");
    request.extensions_mut().insert(ConnectInfo(addr));
    let answer = app.oneshot(request).await.expect("call");
    let status = answer.status();
    let bytes = axum::body::to_bytes(answer.into_body(), 64 * 1024).await.expect("body");
    (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

#[tokio::test]
async fn a_verified_crawler_passes_the_condition_it_otherwise_meets() {
    let (base_url, api) = Api::start(Some("verified")).await;
    let (status, body) = call(app(&base_url, true), Some(GOOGLEBOT)).await;
    assert_eq!(status, StatusCode::OK, "a verified crawler was refused");
    assert_eq!(body, json!({ "attached": true, "verdict": "verified" }));
    assert_eq!(
        *api.seen.lock().unwrap(),
        [(Some(GOOGLEBOT.to_owned()), Some("Bearer key".to_owned()))],
        "the request's own agent and the key reach the API"
    );
}

#[tokio::test]
async fn a_spoofed_or_not_bot_verdict_is_refused() {
    for verdict in ["spoofed", "not_bot"] {
        let (base_url, api) = Api::start(Some(verdict)).await;
        let (status, _) = call(app(&base_url, true), Some(GOOGLEBOT)).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{verdict}");
        assert_eq!(api.agents(), [Some(GOOGLEBOT.to_owned())], "{verdict}");
    }
}

#[tokio::test]
async fn off_sends_no_agent() {
    let (base_url, api) = Api::start(Some("verified")).await;
    let (status, _) = call(app(&base_url, false), Some(GOOGLEBOT)).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "a verdict never asked for let the request through");
    assert_eq!(api.agents(), [None]);
}

#[tokio::test]
async fn a_request_without_a_user_agent_sends_none() {
    let (base_url, api) = Api::start(None).await;
    let (status, _) = call(app(&base_url, true), None).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(api.agents(), [None]);
}

#[tokio::test]
async fn at_most_four_crawler_agents_an_address_reach_the_api() {
    let (base_url, api) = Api::start_verifying_agents().await;
    let app = app(&base_url, true);

    for n in 1..=4 {
        let (status, _) = call(app.clone(), Some(&crawler(n))).await;
        assert_eq!(status, StatusCode::OK, "agent {n}: a verified crawler was refused");
    }
    let (status, _) = call(app.clone(), Some(&crawler(5))).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "a fifth agent was let through as verified");
    let (status, _) = call(app.clone(), Some(&crawler(1))).await;
    assert_eq!(status, StatusCode::OK, "a known agent still passes");
    let (status, _) = call_from(app.clone(), "66.249.66.2", Some(&crawler(5))).await;
    assert_eq!(status, StatusCode::OK, "another address has its own four");

    // The known agent's answer is the client's cached one, so it reaches the API once.
    let sent: Vec<Option<String>> =
        [Some(1), Some(2), Some(3), Some(4), None, Some(5)].map(|n| n.map(crawler)).into();
    assert_eq!(api.agents(), sent);
}

#[tokio::test]
async fn a_browsers_agent_is_not_sent() {
    let (base_url, api) = Api::start_verifying_agents().await;
    let browser = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) \
                   Chrome/151.0.0.0 Safari/537.36";
    let (status, _) = call(app(&base_url, true), Some(browser)).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(api.agents(), [None]);
}

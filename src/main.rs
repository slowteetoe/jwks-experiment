use std::time::Duration;

use axum::Router;
use axum::extract::{Path, State};
use axum::routing::get;

use jwks_client_rs::JwksClient;
use jwks_client_rs::source::WebSource;

use reqwest::Url;

#[derive(Clone)]
struct AppState {
    jwks_client: JwksClient<WebSource>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // using `docker run --rm -d -p 8080:8080 ghcr.io/murar8/local-jwks-server:latest` to start up an example JWKS service
    // curl http://localhost:3000/keys/zodcCJaOzw4xO0yLMagcdpVIA0vIP2dbvCGG7aksO0A to fetch the pre-loaded key
    let url = Url::parse("http://localhost:8080/.well-known/jwks.json").unwrap();

    let timeout: std::time::Duration = Duration::from_millis(500);

    // You can define a different source too using `JwksSource` trait
    let source: WebSource = WebSource::builder()
        .with_timeout(timeout)
        .with_connect_timeout(timeout)
        .build(url)
        .unwrap();

    let jwks_client = JwksClient::builder()
        .time_to_live(Duration::from_secs(10))
        .build(source);

    let app = Router::new()
        .route("/keys/{key_id}", get(key))
        .with_state(AppState {
            jwks_client: jwks_client,
        });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[axum::debug_handler]
async fn key(Path(key_id): Path<String>, State(state): State<AppState>) -> String {
    let result = state.jwks_client.get_opt(&key_id).await;

    if let Ok(Some(keys)) = result {
        format!("{:?}", keys)
    } else {
        "No results found".to_owned()
    }
}

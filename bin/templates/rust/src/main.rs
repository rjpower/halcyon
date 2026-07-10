//! {{name}} — {{description}}
//!
//! Serves `static/` and answers `/healthz` with `ok`. Binds 0.0.0.0 because the
//! only thing that reaches it is halcyon's Caddy, over the shared `web` Docker
//! network; the port is never published to the host.

use std::net::SocketAddr;
use std::path::PathBuf;

use axum::Router;
use axum::routing::get;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let static_dir: PathBuf = std::env::var("{{env_prefix}}_STATIC")
        .unwrap_or_else(|_| "static".into())
        .into();

    let app = Router::new()
        .route("/healthz", get(|| async { "ok" }))
        // Everything the routes above don't claim comes off disk, falling back
        // to index.html so a client-side router owns its own deep links.
        .fallback_service(
            ServeDir::new(&static_dir).fallback(ServeFile::new(static_dir.join("index.html"))),
        )
        .layer(TraceLayer::new_for_http());

    let port: u16 = std::env::var("{{env_prefix}}_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or({{port}});
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    tracing::info!("{{name}} listening on {addr}");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown())
        .await
        .expect("serve");
}

/// `docker stop` sends SIGTERM. Without this the container ignores it and waits
/// out the full ten-second grace period on every single deploy.
async fn shutdown() {
    let mut term = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .expect("install SIGTERM handler");
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {}
        _ = term.recv() => {}
    }
}

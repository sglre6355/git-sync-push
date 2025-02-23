use anyhow::{Ok, Result};
use axum::{extract::State, http::StatusCode, routing::get, Router};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

pub struct AppState {
    pub is_repo_ready: bool,
}

async fn liveness_probe_handler(State(state): State<Arc<Mutex<AppState>>>) -> (StatusCode, String) {
    if !state.lock().await.is_repo_ready {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            "Repository is not ready".to_owned(),
        );
    }

    (StatusCode::OK, "Repository is ready".to_owned())
}

pub async fn serve_health_endpoints(
    http_bind: String,
    shared_state: Arc<Mutex<AppState>>,
) -> Result<()> {
    let app = Router::new()
        .route("/livez", get(liveness_probe_handler))
        .with_state(shared_state);
    let listener = tokio::net::TcpListener::bind(http_bind).await?;

    info!("Serving API endpoints at {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}

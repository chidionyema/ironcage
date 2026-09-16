use axum::{
    body::Body,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    http::StatusCode,
    response::Response,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeDelta {
    pub node_id: String,
    pub visits: usize,
    pub reward: f64,
    pub verified: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResearchClaim {
    pub claim: String,
    pub evidence: Option<String>,
    pub domain: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResearchResult {
    pub claim_id: String,
    pub claim: String,
    pub verified: bool,
    pub evidence_score: f64,
}

pub struct ApiServer {
    tx: broadcast::Sender<NodeDelta>,
}

impl ApiServer {
    pub fn new(buffer_size: usize) -> Self {
        let (tx, _) = broadcast::channel(buffer_size);
        Self { tx }
    }

    pub fn router(self) -> Router {
        let api = Arc::new(self);
        let api_clone = api.clone();
        let api_clone2 = api.clone();

        Router::new()
            .route("/", get(serve_ui))
            .route(
                "/ws",
                get(move |ws: WebSocketUpgrade| {
                    let api = api_clone.clone();
                    async move { ws.on_upgrade(move |socket| handle_ws(socket, api)) }
                }),
            )
            .route("/research/claim", post(submit_claim))
            .route("/health", get(|| async { "ok" }))
            .route(
                "/metrics",
                get(move || {
                    let _api = api_clone2.clone();
                    async move { "# Ironcage Metrics\nironcage_api_version{} 1\n" }
                }),
            )
            .layer(CorsLayer::permissive())
    }

    pub fn broadcast_delta(&self, delta: NodeDelta) {
        let _ = self.tx.send(delta);
    }

    pub fn tx(&self) -> &broadcast::Sender<NodeDelta> {
        &self.tx
    }
}

async fn submit_claim(Json(claim): Json<ResearchClaim>) -> Json<ResearchResult> {
    let claim_id = format!("claim-{}", Uuid::new_v4());
    Json(ResearchResult {
        claim_id: claim_id.clone(),
        claim: claim.claim,
        verified: false,
        evidence_score: 0.0,
    })
}

async fn serve_ui() -> Result<Response, StatusCode> {
    let html = include_str!("../../../ui/index.html");
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html")
        .body(Body::from(html))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn handle_ws(mut socket: WebSocket, api: Arc<ApiServer>) {
    let mut rx = api.tx().subscribe();

    loop {
        tokio::select! {
            result = rx.recv() => {
                match result {
                    Ok(delta) => {
                        let json_str = serde_json::to_string(&delta).unwrap();
                        let msg = Message::Text(json_str.into());
                        if socket.send(msg).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Close(_))) => break,
                    Some(Ok(Message::Text(t))) => {
                        tracing::debug!("Received: {}", t);
                    }
                    Some(Err(_)) => break,
                    None => break,
                    _ => {}
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_creation() {
        let api = ApiServer::new(100);
        let _router = api.router();
    }

    #[test]
    fn test_node_delta_serialization() {
        let delta = NodeDelta {
            node_id: "test-id".to_string(),
            visits: 10,
            reward: 0.85,
            verified: true,
        };
        let json = serde_json::to_string(&delta).unwrap();
        assert!(json.contains("test-id"));
    }
}

use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, HeaderValue, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::get,
    Json, Router,
};
use rockcode_protocol::{
    CapabilitiesResp, ErrorBody, ErrorResp, ExecutorCapability, HealthResp, ReadinessCheck,
    ReadyResp,
};
use uuid::Uuid;

async fn health_live() -> Json<HealthResp> {
    Json(HealthResp {
        status: "live".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn health_ready() -> Json<ReadyResp> {
    Json(ReadyResp {
        status: "ready".to_string(),
        checks: vec![ReadinessCheck {
            name: "http".to_string(),
            status: "ok".to_string(),
            detail: Some("HTTP server is accepting requests".to_string()),
        }],
    })
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }

    left.iter()
        .zip(right)
        .fold(0_u8, |difference, (a, b)| difference | (a ^ b))
        == 0
}

fn is_authorized(header: Option<&HeaderValue>, token: &str) -> bool {
    if token.len() < 32 {
        return false;
    }

    let Some(value) = header.and_then(|value| value.to_str().ok()) else {
        return false;
    };
    let expected = format!("Bearer {token}");

    constant_time_eq(value.as_bytes(), expected.as_bytes())
}

fn unauthorized() -> (StatusCode, Json<ErrorResp>) {
    (
        StatusCode::UNAUTHORIZED,
        Json(ErrorResp {
            error: ErrorBody {
                code: "unauthenticated".to_string(),
                message: "A valid RockCode bearer token is required.".to_string(),
                retryable: false,
                request_id: format!("req_{}", Uuid::new_v4().simple()),
            },
        }),
    )
}

async fn auth_middleware(
    State(token): State<Arc<str>>,
    req: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<ErrorResp>)> {
    let path = req.uri().path();
    if matches!(path, "/health/live" | "/health/ready") {
        return Ok(next.run(req).await);
    }

    if is_authorized(req.headers().get(AUTHORIZATION), token.as_ref()) {
        return Ok(next.run(req).await);
    }

    Err(unauthorized())
}

async fn capabilities() -> Json<CapabilitiesResp> {
    Json(CapabilitiesResp {
        server_version: env!("CARGO_PKG_VERSION").to_string(),
        minimum_client_version: "0.1.0".to_string(),
        protocol_versions: vec!["v1".to_string()],
        event_schema_versions: vec![1],
        features: vec!["health".to_string(), "capabilities".to_string()],
        executors: vec![ExecutorCapability {
            kind: "local".to_string(),
            available: false,
            limitations: vec![
                "Workspace execution is not enabled in this build.".to_string(),
                "No filesystem or shell tools are exposed by the server yet.".to_string(),
            ],
        }],
    })
}

pub fn app() -> Router {
    let token = std::env::var("ROCKCODE_API_TOKEN").unwrap_or_default();
    app_with_token(token)
}

pub fn app_with_token(token: impl Into<String>) -> Router {
    Router::new()
        .route("/health/live", get(health_live))
        .route("/health/ready", get(health_ready))
        .route("/capabilities", get(capabilities))
        .layer(middleware::from_fn_with_state(
            Arc::<str>::from(token.into()),
            auth_middleware,
        ))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_TOKEN: &str = "rockcode-test-token-00000000000000000000000000000000";

    #[test]
    fn accepts_only_the_exact_bearer_token() {
        let valid = HeaderValue::from_str(&format!("Bearer {TEST_TOKEN}")).unwrap();
        let wrong = HeaderValue::from_static("Bearer wrong-token");

        assert!(is_authorized(Some(&valid), TEST_TOKEN));
        assert!(!is_authorized(Some(&wrong), TEST_TOKEN));
        assert!(!is_authorized(None, TEST_TOKEN));
    }

    #[test]
    fn rejects_missing_or_weak_server_tokens() {
        let valid = HeaderValue::from_str(&format!("Bearer {TEST_TOKEN}")).unwrap();

        assert!(!is_authorized(Some(&valid), ""));
        assert!(!is_authorized(Some(&valid), "short"));
    }

    #[test]
    fn compares_equal_length_tokens_without_early_byte_exit() {
        assert!(constant_time_eq(b"same-length", b"same-length"));
        assert!(!constant_time_eq(b"same-length", b"same-leng th"));
        assert!(!constant_time_eq(b"short", b"longer"));
    }
}

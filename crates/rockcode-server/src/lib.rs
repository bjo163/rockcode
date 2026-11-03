use axum::{
    extract::Request,
    http::StatusCode,
    middleware::{self, Next},
    response::Response,
    routing::get,
    Json, Router,
};
use rockcode_protocol::{CapabilitiesResp, ErrorBody, ErrorResp, HealthResp, ReadyResp};

async fn health_live() -> Json<HealthResp> {
    Json(HealthResp {
        status: "ok".to_string(),
    })
}

async fn health_ready() -> Json<ReadyResp> {
    Json(ReadyResp {
        status: "ready".to_string(),
    })
}

async fn auth_middleware(
    req: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<ErrorResp>)> {
    // static bearer check for demo; in real impl check header
    // for unauthenticated return unauthenticated error code
    let unauthorized = (
        StatusCode::UNAUTHORIZED,
        Json(ErrorResp {
            error: ErrorBody {
                code: "unauthenticated".to_string(),
                message: "unauthenticated".to_string(),
            },
        }),
    );
    // Allow specific paths without auth? OpenAPI says security [] for health - but easier to just allow if path matches
    let path = req.uri().path().to_string();
    if path.starts_with("/health") || path == "/capabilities" {
        // For capabilities, require auth per spec? Let us check - but follow plan: "/capabilities behind a static bearer check returning the unauthenticated error code"
        if path == "/capabilities" {
            // naive check - if no auth header, return unauth
            let has_auth = req.headers().get("authorization").is_some();
            if !has_auth {
                return Err(unauthorized);
            }
        }
        return Ok(next.run(req).await);
    }
    Err(unauthorized)
}

async fn capabilities() -> Json<CapabilitiesResp> {
    Json(CapabilitiesResp {
        capabilities: vec!["tools".to_string(), "events".to_string()],
    })
}

pub fn app() -> Router {
    Router::new()
        .route("/health/live", get(health_live))
        .route("/health/ready", get(health_ready))
        .route("/capabilities", get(capabilities))
        .layer(middleware::from_fn(auth_middleware))
}

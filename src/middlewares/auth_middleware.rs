use axum::{body::Body, extract::{Request, State}, http::StatusCode, middleware::Next, response::{IntoResponse, Response}};

use crate::AppState;

pub async fn auth_middleware(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|header| header.to_str().ok());

    match auth_header {
        Some(token) if is_valid_token(token) => {
            Ok(next.run(req).await)
        }
        _ => {
            Err((
                StatusCode::UNAUTHORIZED,
                "Invalid or missing authorization token".to_string(),
            ))
        }
    }
}

fn is_valid_token(token: &str) -> bool {
    token.starts_with("Bearer ") && token.len() > 10
}

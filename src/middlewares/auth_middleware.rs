use axum::{body::Body, extract::{Request, State}, http::StatusCode, middleware::Next, response::Response};
use mongodb::{bson::doc, Collection};
use tower_cookies::Cookies;

use crate::{models::auth_schema::Sessions, AppState};

pub async fn auth_middleware(
    State(state): State<AppState>,
    cookies: Cookies,
    req: Request<Body>,
    next: Next,
) -> Result<Response<Body>, (StatusCode, String)> {

    let session_id_value = match cookies.get("session_id") {
        Some(cookie) => cookie.value().to_string(),
        None => return Err((StatusCode::UNAUTHORIZED, "Missing session_id cookie".to_string())),
    };

    println!("Session ID: {}", session_id_value);

    let session_coll: Collection<Sessions> = state.db.collection("sessions");

    match session_coll.find_one(doc! {"session_id": session_id_value}).await {
        Ok(Some(_session)) => {
            Ok(next.run(req).await)
        }
        Ok(None) => {
            Err((StatusCode::UNAUTHORIZED, "Session not found".to_string())) 
        }
        Err(e) => {
            println!("Database error: {:?}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())) 
        }
    }
}
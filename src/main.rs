use axum::{
    http::StatusCode, response::IntoResponse, routing::{get, post}, Json, Router
};

use mongodb::{bson::doc, Collection};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

mod database;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(simple))
                            .route("/dynamic", post(dynamic_payload))
                            .route("/login", post(login));

    const PORT:u16 = 3000;
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{:?}",PORT)).await.unwrap();
    println!("Server started at http://localhost:{:?}", PORT);
    axum::serve(listener, app).await.unwrap();
}


#[derive(Deserialize, Debug, Serialize)]
struct LoginPayload {
    email: String,
    password: String,
}

async fn login(Json(payload): Json<LoginPayload>) -> impl IntoResponse {
    let db = database::connect("mongodb://localhost:27017/", "Tourney")
        .await
        .expect("Failed to connect to the database");
    println!("Received payload: {:?}", payload);
    
    let my_coll:Collection<LoginPayload> = db.collection("users");

    match my_coll.find_one(doc! { "email": &payload.email }).await {
        Ok(Some(user)) => {
            if payload.password == user.password {
                (StatusCode::OK, "Success").into_response()
            } else {
                (StatusCode::UNAUTHORIZED, "Invalid password").into_response()
            }
        },
        Ok(None) => {
            (StatusCode::NOT_FOUND, "User not found").into_response()
        },
        Err(e) => {
            println!("Database error: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
        }
    }

}

async fn dynamic_payload(Json(payload): Json<Value>) -> impl IntoResponse {
    if let Some(username) = payload.get("username").and_then(|v| v.as_str()) {
        let response = json!({
            "message": format!("Received data for user: {}", username),
            "data": payload
        });
        (StatusCode::OK, Json(response))
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(json!(payload.get("email"))),
        )
    }
}

async fn simple() -> impl IntoResponse {
    "Success"
}
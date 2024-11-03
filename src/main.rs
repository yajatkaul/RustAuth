use axum::{
    extract::State, http::StatusCode, response::IntoResponse, routing::{get, post}, Json, Router
};

use mongodb::{bson::doc, Collection, Database};
use serde::{Deserialize, Serialize};

mod database;

#[derive(Clone)]
struct AppState {
    db: Database,
}

#[tokio::main]
async fn main() {

    //DB connection
    let db = database::connect("mongodb://localhost:27017/", "Tourney")
    .await
    .expect("Failed to connect to the database");
    
    let state = AppState { db };

    //Server Setup
    let app = Router::new().route("/login", post(login))
                                   .route("/signup", post(signup)).with_state(state);

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

async fn login(State(state): State<AppState>, Json(payload): Json<LoginPayload>) -> impl IntoResponse {
    println!("Received Login payload: {:?}", payload);
    
    let my_coll:Collection<LoginPayload> = state.db.collection("users");

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

#[derive(Deserialize, Debug, Serialize)]
struct SignupPayload {
    #[serde(rename(serialize = "user_name", deserialize = "user_name"))]
    user_name: String,
    #[serde(rename(serialize = "display_name", deserialize = "display_name"))]
    display_name: String,
    #[serde(rename(serialize = "email", deserialize = "email"))]
    email: String,
    #[serde(rename(serialize = "password", deserialize = "password"))]
    password: String,
}

async fn signup(State(state): State<AppState>, Json(payload): Json<SignupPayload>) -> impl IntoResponse {
    println!("Received Signup payload: {:?}", payload);
    
    let my_coll:Collection<SignupPayload> = state.db.collection("users");

    match my_coll.find_one(doc! { "email": &payload.email }).await {
        Ok(Some(_user)) => {
            (StatusCode::BAD_REQUEST, "Email already used").into_response()
        },
        Ok(None) => {
            let user = SignupPayload {
                user_name: payload.user_name,
                display_name: payload.display_name,
                email: payload.email,
                password: payload.password,
            };

            let _ = my_coll.insert_one(&user).await;

            (StatusCode::OK, "Account created sucessfully").into_response()
        },
        Err(e) => {
            println!("Database error: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
        }
    }
}
#![deny(elided_lifetimes_in_paths)]

use axum::{
    extract::State, http::{header, HeaderMap, HeaderValue, StatusCode}, response::IntoResponse, routing::{get, post}, Json, Router
};
use tower_cookies::{Cookie, CookieManagerLayer, Cookies};

use chrono::{DateTime, Duration, Utc};
use mongodb::{bson::{doc, oid::ObjectId}, Collection, Database};
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use serde_json::json;

use bcrypt::{hash_with_salt, verify};

use uuid::Uuid;

mod database;


#[derive(Clone)]
struct AppState {
    db: Database,
}

#[tokio::main]
async fn main() {

    //DB connection
    let db = database::connect("mongodb://localhost:27017/", "rustier")
    .await
    .expect("Failed to connect to the database");
    
    let state = AppState { db };

    //Server Setup
    let app = Router::new().route("/login", post(login))
                                   .route("/logout", get(logout))
                                   .route("/signup", post(signup))
                                   .with_state(state)
                                   .layer(CookieManagerLayer::new());

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

#[derive(Deserialize, Debug, Serialize)]
struct Sessions {
    session_id: String,
    user_id: String,
    valid_till: DateTime<Utc>,
}

#[derive(Deserialize, Debug, Serialize)]
struct UserSchema {
    #[serde(rename = "_id")]
    id: ObjectId,
    email: String,
    user_name: String,
    password: String,
}

async fn login(State(state): State<AppState>, Json(payload): Json<LoginPayload>) -> impl IntoResponse {
    println!("Received Login payload: {:?}", payload);
    
    let my_coll:Collection<UserSchema> = state.db.collection("users");

    match my_coll.find_one(doc! { "email": &payload.email }).await {
        Ok(Some(user)) => {
            let verification = verify(payload.password, &user.password).expect("Error while verifying");
            if verification {
                let sessions_coll:Collection<Sessions> = state.db.collection("sessions");

                let id = Uuid::new_v4();

                let exp = Utc::now() + Duration::days(7);

                let session = Sessions {
                    session_id: id.to_string(),
                    user_id: user.id.to_string(),
                    valid_till: exp,
                };

                let _ = sessions_coll.insert_one(&session).await;
                let mut headers = HeaderMap::new();

                headers.insert(
                    header::SET_COOKIE,
                    HeaderValue::from_str(&format!("session_id={:?}; Path=/; HttpOnly; Expires={};", id, exp.to_rfc2822())).unwrap(),
                );

                (StatusCode::OK,headers, "Logged in successfully").into_response()
            } else {
                (StatusCode::UNAUTHORIZED, "Username or Password Incorrect").into_response()
            }
        },
        Ok(None) => {
            (StatusCode::NOT_FOUND, "Username or Password Incorrect").into_response()
        }, 
        Err(e) => {
            println!("Database error: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
        }
    }

}

#[derive(Deserialize, Debug, Serialize)]
struct SignupPayload {
    user_name: String,
    email: String,
    password: String,
}


async fn signup(State(state): State<AppState>, Json(payload): Json<SignupPayload>) -> impl IntoResponse {
    println!("Received Signup payload: {:?}", payload);
    
    let my_coll:Collection<UserSchema> = state.db.collection("users");

    match my_coll.find_one(doc! { "email": &payload.email }).await {
        Ok(Some(_user)) => {
            (StatusCode::BAD_REQUEST, Json(json!({"result": "Email already used"}))).into_response()
        },
        Ok(None) => {
            let mut salt = [0u8; 16];
            OsRng.fill_bytes(&mut salt);

            let id = ObjectId::new();

            let sessions_coll:Collection<Sessions> = state.db.collection("sessions");

            match hash_with_salt(payload.password, 10, salt) {
                Ok(hash) => {
                    let user = UserSchema {
                        id: id,
                        user_name: payload.user_name,
                        email: payload.email,
                        password: hash.to_string(),
                    };
                    
                    let _ = my_coll.insert_one(&user).await;
                }
                Err(_e) => {
                    (StatusCode::BAD_REQUEST, "Error hashing password").into_response();
                },
            };
            
            let session_id  = Uuid::new_v4();

            let exp = Utc::now() + Duration::days(7);
            let session = Sessions{
                session_id: session_id.to_string(),
                user_id: id.to_string(),
                valid_till: exp,
            };

            let _ = sessions_coll.insert_one(session);

            let mut headers = HeaderMap::new();

            headers.insert(
                header::SET_COOKIE,
                HeaderValue::from_str(&format!("session_id={:?}; Path=/; HttpOnly; Expires={};", id, exp.to_rfc2822())).unwrap(),
            );

            (StatusCode::OK,headers, "Account created sucessfully").into_response()
        },
        Err(e) => {
            println!("Database error: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
        }
    }
}

async fn logout(State(state): State<AppState>, cookies: Cookies) -> impl IntoResponse {
    match cookies.get("session") {
        Some(cookie) => {
            println!("Found session: {}", cookie.value());
            cookies.remove(cookie);
            (StatusCode::OK, "Logged out successfully").into_response()
        }
        None => (StatusCode::BAD_REQUEST, "No session found").into_response()
    }
}
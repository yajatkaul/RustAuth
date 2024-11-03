use axum::{
    extract::State, http::{header, HeaderMap, HeaderValue, StatusCode}, response::IntoResponse, Json
};
use tower_cookies::{Cookie, Cookies};

use chrono::{Duration, Utc};
use mongodb::{bson::{doc, oid::ObjectId}, Collection};
use rand::{rngs::OsRng, RngCore};
use serde_json::json;

use bcrypt::{hash_with_salt, verify};

use uuid::Uuid;

use crate::{models::{auth_schema::{Sessions, UserSchema}, route_schema::{LoginPayload, SignupPayload}}, AppState};

pub async fn login(State(state): State<AppState>, Json(payload): Json<LoginPayload>) -> impl IntoResponse {
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
                    HeaderValue::from_str(&format!("session_id={}; Path=/; HttpOnly; Expires={};", id, exp.to_rfc2822())).unwrap(),
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

pub async fn signup(State(state): State<AppState>, Json(payload): Json<SignupPayload>) -> impl IntoResponse {
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
                    return (StatusCode::BAD_REQUEST, "Error hashing password").into_response();
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
                HeaderValue::from_str(&format!("session_id={}; Path=/; HttpOnly; Expires={};", session_id, exp.to_rfc2822())).unwrap(),
            );

            (StatusCode::OK,headers, "Account created sucessfully").into_response()
        },
        Err(e) => {
            println!("Database error: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
        }
    }
}

pub async fn logout(State(state): State<AppState>, cookies: Cookies) -> impl IntoResponse {
    match cookies.get("session_id") {
        Some(cookie) => {
            let sessions_coll:Collection<Sessions> = state.db.collection("sessions");
            println!("{:?}", cookie.value());
            let _ = sessions_coll.delete_one(doc! {"session_id": cookie.value()}).await;
            cookies.remove(Cookie::new("session_id", ""));
            (StatusCode::OK, "Logged out successfully")
        }
        None => (StatusCode::BAD_REQUEST, "No session found")
    }.into_response()
}
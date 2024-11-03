use axum::{routing::{get, post}, Router};
use mongodb::Database;
use tower_cookies::CookieManagerLayer;
use utils::multer::upload_file_handler;

mod database;
mod models;
mod controllers;
mod utils;

use crate::controllers::auth_controllers::{login, logout, signup};

#[derive(Clone)]
struct AppState {
    db: Database,
}

#[tokio::main]
async fn main() {
    //DB connection
    let db = database::mongo::connect("mongodb://localhost:27017/", "rustier")
    .await
    .expect("Failed to connect to the database");
    
    let state = AppState { db };
    
    //Server Setup
    let app = Router::new()
                                   //Auth routes
                                   .route("/login", post(login))
                                   .route("/logout", get(logout))
                                   .route("/signup", post(signup))

                                   //Upload routes
                                   .route("/upload", post(upload_file_handler))
                                   
                                   .with_state(state)
                                   .layer(CookieManagerLayer::new());


    const PORT:u16 = 3000;
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{:?}",PORT)).await.unwrap();
    println!("Server started at http://localhost:{:?}", PORT);
    axum::serve(listener, app).await.unwrap();
}


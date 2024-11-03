use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize)]
pub struct LoginPayload {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct SignupPayload {
    pub user_name: String,
    pub email: String,
    pub password: String,
}
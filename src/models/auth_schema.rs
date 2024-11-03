use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize)]
pub struct Sessions {
    pub session_id: String,
    pub user_id: String,
    pub valid_till: DateTime<Utc>,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct UserSchema {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub email: String,
    pub user_name: String,
    pub password: String,
}
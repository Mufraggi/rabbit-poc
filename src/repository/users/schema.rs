use mongodb::bson::DateTime;
use mongodb::bson::oid::ObjectId;
use serde::Serialize;
use serde::Deserialize;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub age: i32,
    #[serde(rename = "createdAt")]
    pub created_at: Option<DateTime>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<DateTime>,
}
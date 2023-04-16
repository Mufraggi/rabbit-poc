use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct UsersRabbit {
    pub name: String,
    pub age: i32,
}
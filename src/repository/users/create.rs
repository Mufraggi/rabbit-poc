use mongodb::{bson, Client, Collection};
use mongodb::bson::{doc, Document};
use mongodb::bson::oid::ObjectId;
use mongodb::options::{ClientOptions, FindOneOptions};
use crate::repository::users::schema::User;
use async_trait::async_trait;
use futures::future::err;
use mongodb::error::Error;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone)]
pub struct MongoRepository {
    collection: Collection<Document>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub enum InsertError {
    Conflict,
    Unknown,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub enum FetchOneError {
    NotFound,
    Unknown,
}

impl MongoRepository {
    pub async fn new(url_db: &str) -> Result<MongoRepository, mongodb::error::Error> {
        let client_options =
            ClientOptions::parse("mongodb://localhost:27017").await;
        match client_options {
            Ok(options) => {
                match Client::with_options(options) {
                    Ok(client) => {
                        let db = client.database("mydb");
                        let user_collection: Collection<Document> = db.collection("users");
                        Ok(MongoRepository { collection: user_collection })
                    }
                    Err(e) => Err(e)
                }
            }
            Err(err) => {
                Err(err)
            }
        }
    }
}

#[async_trait]
pub trait Repository {
    async  fn insert(&self ,user: &User) ->  anyhow::Result<ObjectId, InsertError>;
    async fn find_by_id(&self, id: ObjectId) -> anyhow::Result<User, FetchOneError>;
}



#[async_trait]
impl Repository for MongoRepository {
    async fn insert(&self ,user: &User) ->  anyhow::Result<ObjectId, InsertError> {
        match bson::to_document(user) {
            Ok(doc) => {
                match self.collection.insert_one(doc, None).await {
                    Ok(document) => {
                        Ok(document.inserted_id.as_object_id().unwrap())
                    }
                    Err(_) => {Err(InsertError::Conflict)}
                }
            }
            Err(_) => {
                Err(InsertError::Unknown)
            }
        }
    }

    async fn find_by_id(&self, id: ObjectId) -> anyhow::Result<User, FetchOneError> {
        let filter = doc! {"_id": id};
        let options = FindOneOptions::builder().build();
        match  self.collection.find_one(filter, options).await {
            Ok(Some(doc)) => {
                let user: User = bson::from_document(doc).unwrap();
                Ok(user)
            },
            Err(e) => {Err(FetchOneError::Unknown)}
            _ => { Err(FetchOneError::NotFound )}
        }
    }
}

#[cfg(test)]
mod tests {
    use mongodb::bson::DateTime;
    use mongodb::bson::oid::ObjectId;
    use crate::repository::users::create::{MongoRepository, Repository};
    use crate::repository::users::schema::User;

    #[tokio::test]
    async fn create_works() {
        let utc = DateTime::now();
        let user = User {
            id: Some(ObjectId::new()),
            name: "muf".to_string(),
            age: 12,
            created_at: Some(utc),
            updated_at: None,
        };
        let mongo_repo = MongoRepository::new("aaa").await.unwrap();
        let res_id = mongo_repo.insert(&user).await.unwrap();

        assert_eq!(res_id, user.id.unwrap())
    }

    #[tokio::test]
    async fn find_one_works() {
        let utc = DateTime::now();
        let user = User {
            id: Some(ObjectId::new()),
            name: "muf".to_string(),
            age: 12,
            created_at: Some(utc),
            updated_at: None,
        };
        let user_res = User {
            id: user.id.clone(),
            name: user.name.clone(),
            age:12,
            created_at:Some(utc),
            updated_at: None,
        };
        let mongo_repo = MongoRepository::new("aaa").await.unwrap();
        let res_id = mongo_repo.insert(&user).await.unwrap();
        let res = mongo_repo.find_by_id(res_id).await.unwrap();
        assert_eq!(res, user_res)
    }
}
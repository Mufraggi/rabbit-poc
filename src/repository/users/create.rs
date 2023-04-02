use mongodb::{bson, Client, Collection};
use mongodb::bson::Document;
use mongodb::bson::oid::ObjectId;
use mongodb::options::ClientOptions;
use crate::repository::users::schema::User;
use async_trait::async_trait;



#[derive(Clone)]
pub struct MongoRepository {
    collection: Collection<Document>,
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
    async  fn insert(&self ,user: &User) ->  anyhow::Result<ObjectId, mongodb::error::Error>;
}



#[async_trait]
impl Repository for MongoRepository {
    async fn insert(&self ,user: &User) ->  anyhow::Result<ObjectId, mongodb::error::Error> {
        let result = self.collection.insert_one(bson::to_document(user)?, None).await?;

        Ok(result.inserted_id.as_object_id().unwrap().clone())
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
        let user_res = User {
            id: user.id.clone(),
            name: user.name.clone(),
            age:12,
            created_at:Some(utc),
            updated_at: None,
        };
        let mongo_repo = MongoRepository::new("aaa").await.unwrap();
        let res_id = mongo_repo.insert(&user).await.unwrap();

        assert_eq!(res_id, user.id.unwrap())
    }
}
mod api;
mod rabbit;
mod repository;

use std::cell::RefCell;
use std::future::Future;
use crate::api::serve;
use crate::rabbit::rabbit_builder::{QueueDefinition, RabbitHandlerBuilder};
use futures::join;
use lapin::message::Delivery;
use lapin::options::QueueDeclareOptions;
use lapin::types::FieldTable;
use mongodb::bson::DateTime;
use mongodb::bson::oid::ObjectId;
use crate::rabbit::interfaces::users_rabbit::UsersRabbit;
use crate::repository::users;
use crate::repository::users::schema::User;
use crate::repository::users::users_repository::{MongoRepository, Repository};
use serde::{Serialize, Deserialize};
use crate::rabbit::generic_function::FuncGeneric;

//use crate::rabbit::{RabbitHandler, rmq_listen};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("{}",1);
    let actix_thread = serve("localhost");
    println!("{}",2);
    let mut rabbit_handler = RabbitHandlerBuilder::new()
        .addr(
            std::env::var("AMQP_ADDR")
                .unwrap_or_else(|_| "amqp://rmq:rmq@127.0.0.1:5672/%2f".into()),
        )
        .push_queue_definition(QueueDefinition {
            name: "hello".to_string(),
            options: QueueDeclareOptions::default(),
            args: FieldTable::default(),
            func: FuncGeneric::new(move |   x: Delivery| {
                Box::pin(async move   {
                    create_user_from_rabbit(
                        MongoRepository::new("aa").await.unwrap(),
                        x,
                    ).await
                })
            }),
        })
        /* .push_queue_definition(QueueDefinition{
             name: "hello22".to_string(),
             options: QueueDeclareOptions::default(),
             args: FieldTable::default(),
             func: FuncGeneric::new(move |b| {
                 Box::pin(async move {
                     add(5, b).await
                 })}),
         })*/
        .build()
        .unwrap();
    let rmq_thread = rabbit_handler.listen_rmq();
    let _ = join!(
     actix_thread,
    rmq_thread
    );
    Ok(())
}


async fn create_user_from_rabbit(mongo_repo: MongoRepository, delivery: Delivery) {
    let message = std::str::from_utf8(&delivery.data).unwrap();
    println!("{}", message);
    let my_message: UsersRabbit = serde_json::from_str(message).unwrap();
    println!("{:?}", my_message);
    let mongo_user: User = User {
        id: Some(ObjectId::new()),
        name: my_message.name,
        age: my_message.age,
        created_at: Some(DateTime::now()),
        updated_at: None,
    };
    let res = mongo_repo.insert(&mongo_user).await;
}

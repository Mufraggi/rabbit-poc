use deadpool_lapin::{Config, Manager, Pool, PoolError};
use lapin::options::{BasicAckOptions, BasicConsumeOptions, QueueDeclareOptions};
use lapin::types::FieldTable;
use lapin::ConnectionProperties;
use mongodb::bson::oid::ObjectId;
use mongodb::bson::DateTime;
use std::time::Duration;
use tokio_amqp::LapinTokioExt;
use thiserror::Error as ThisError;
use std::result::Result as StdResult;
use futures::StreamExt;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("rmq error: {0}")]
    RMQError(#[from] lapin::Error),
    #[error("rmq pool error: {0}")]
    RMQPoolError(#[from] PoolError),
}

type Result<T> = StdResult<T, Error>;
type RMQResult<T> = StdResult<T, PoolError>;

type Connection = deadpool::managed::Object<Manager>;

pub fn init_rabbit() -> Pool {
    let addr =
        std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://rmq:rmq@127.0.0.1:5672/%2f".into());
    let manager = Manager::new(addr, ConnectionProperties::default().with_tokio());
    let pool: Pool = Pool::builder(manager)
        .max_size(10)
        .build()
        .expect("can create pool");
    return pool;
    let rmq_thread = rmq_listen(pool.clone());
}

pub async fn rmq_listen(pool: Pool) -> Result<()> {
    let mut retry_interval = tokio::time::interval(Duration::from_secs(5));
    loop {
        retry_interval.tick().await;
        println!("connecting rmq consumer...");
        match init_rmq_listen(pool.clone()).await {
            Ok(_) => println!("rmq listen returned"),
            Err(e) => eprintln!("rmq listen had an error: {}", e),
        };
    }
}

async fn get_rmq_con(pool: Pool) -> RMQResult<Connection> {
    let connection = pool.get().await?;
    Ok(connection)
}

async fn init_rmq_listen(pool: Pool) -> Result<()> {
    let rmq_con = get_rmq_con(pool).await.map_err(|e| {
        eprintln!("could not get rmq con: {}", e);
        e
    })?;
    let channel = rmq_con.create_channel().await?;

    let queue = channel
        .queue_declare(
            "hello",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;
    println!("Declared queue {:?}", queue);
    let queue2 = channel
        .queue_declare(
            "hello2",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;
    println!("Declared queue {:?}", queue2);

    let mut consumer = channel
        .basic_consume(
            "hello",
            "my_consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    println!("rmq consumer connected, waiting for messages");
    while let Some(delivery) = consumer.next().await {
        match delivery {
            Ok(delivery) => {
                // let message = std::str::from_utf8(&delivery.data).unwrap();
                //let my_message: MyMessage = serde_json::from_str(message).unwrap();
               /* let utc = DateTime::now();
                let user = User {
                    id: Some(ObjectId::new()),
                    name: "muuuuuuuuuuf".to_string(),
                    age: 12,
                    created_at: Some(utc),
                    updated_at: None,
                };
                let q = users_repository::MongoRepository::new("")
                    .await
                    .unwrap();
                q.insert(&user).await;*/
                println!(
                    "received msg: {:?}",
                    String::from_utf8_lossy(&delivery.data)
                );
                channel
                    .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
                    .await?
            }
            Err(_) => (()),
        }
    }
    Ok(())
}

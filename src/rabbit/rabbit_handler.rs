use crate::rabbit::rabbit_builder::QueueDefinition;
use deadpool_lapin::{Manager, Pool, PoolError};
use lapin::Channel;
use std::result::Result as StdResult;
use std::time::Duration;
use futures::StreamExt;
use lapin::options::{BasicAckOptions, BasicConsumeOptions};
use lapin::types::FieldTable;
use thiserror::Error as ThisError;
type Result<T> = StdResult<T, Error>;
type RMQResult<T> = StdResult<T, PoolError>;

type Connection = deadpool::managed::Object<Manager>;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("rmq error: {0}")]
    RMQError(#[from] lapin::Error),
    #[error("rmq pool error: {0}")]
    RMQPoolError(#[from] PoolError),
}

#[derive(Debug)]
pub struct RabbitHandler {
    pub rabbit_pool: Pool,
    pub queue_definition: Vec<QueueDefinition>,
}

impl RabbitHandler {
    pub async fn listen_rmq(&self) -> RabbitHandler {
        let mut retry_interval = tokio::time::interval(Duration::from_secs(5));
        loop {
            retry_interval.tick().await;
            println!("connecting rmq consumer...");
            match self.init_rmq_listen().await {
                Ok(_) => println!("rmq listen returned"),
                Err(e) => eprintln!("rmq listen had an error: {}", e),
            };
        }
    }
    async fn init_rmq_listen(&self) -> Result<()> {
        let rmq_con = self.get_rmq_con().await.map_err(|e| {
            eprintln!("could not get rmq con: {}", e);
            e
        })?;
        let channel: Channel = rmq_con.create_channel().await?;
        for QueueDefinition {
            name,
            options,
            args,
        } in self.queue_definition.iter() {
            let queue = channel.queue_declare(&*name, *options, args.clone()).await?;
        }
        /*self.queue_definition.iter().map(
            |QueueDefinition {
                 name,
                 options,
                 args,
             }| {
                println!("{}", name);
                let queue = channel.queue_declare(name, *options, args.clone());
                async {
                    match queue.await {
                        Ok(res) => {
                            println!("Declared queue {:?}", res);
                        }
                        Err(_) => (())
                    }
                }
            },
        );*/
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

    async fn get_rmq_con(&self) -> RMQResult<Connection> {
        let connection = self.rabbit_pool.clone().get().await?;
        Ok(connection)
    }
}

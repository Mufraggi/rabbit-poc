use std::ops::Deref;
use crate::rabbit::rabbit_builder::QueueDefinition;
use deadpool_lapin::{Manager, Pool, PoolError};
use futures::future::ok;
use futures::future::Future;
use futures::StreamExt;
use lapin::message::Delivery;
use lapin::options::{BasicAckOptions, BasicConsumeOptions};
use lapin::types::FieldTable;
use lapin::Channel;
use std::pin::Pin;
use std::result::Result as StdResult;
use std::slice::Iter;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;
use thiserror::Error as ThisError;
use tokio::task;
use crate::rabbit::generic_function::FuncGeneric;


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

pub struct RabbitHandler<
    F: Fn(Delivery) -> Pin<Box<(dyn futures::Future<Output = ()> + 'static)>> + 'static,
> {
    pub rabbit_pool: Pool,
    pub queue_definition: Vec<QueueDefinition<F>>,
}

impl<F: Fn(Delivery) -> Pin<Box<(dyn futures::Future<Output = ()> + 'static)>> + 'static> Clone
    for RabbitHandler<F>
{
    fn clone(&self) -> Self {
        self.clone()
    }
}

impl<F: Fn(Delivery) -> Pin<Box<(dyn Future<Output = ()> + 'static)>> + std::marker::Sync>
    RabbitHandler<F>
{
    pub async fn listen_rmq<'a>(&'a mut self) -> RabbitHandler<F>
    where
        F: 'static,
    {
        println!("{}",11);
        let mut retry_interval = tokio::time::interval(Duration::from_secs(5));
        println!("{}",12);
        loop {
            retry_interval.tick().await;
            println!("connecting rmq consumer...");
            let rmq_con: Connection = self
                .get_rmq_con()
                .await
                .map_err(|e| {
                    eprintln!("could not get rmq con: {}", e);
                    e
                })
                .unwrap();
            let def = &self.queue_definition.iter();
            let _ = init_rmq_listen(rmq_con, def).await;
        }
    }

    async fn get_rmq_con(&self) -> RMQResult<Connection> {
        let connection = self.rabbit_pool.clone().get().await?;
        Ok(connection)
    }
}

async fn consume_messages<
    'a,
    F: Fn(Delivery) -> Pin<Box<(dyn Future<Output = ()> + 'static)>> + std::marker::Sync,
>(
    channel: Channel,
    queue: &str,
    consumer_tag: &str,
    definition: &FuncGeneric<F>,
) {
    let mut consumer = channel
        .basic_consume(
            queue,
            consumer_tag,
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .expect("Erreur de consommation de la file d'attente");

    loop {
        let (Some(Ok(a)), _) = consumer.clone().into_future().await else { panic!("{}","eroor");
        };
        definition.call(a).await;
    }
}

async fn init_rmq_listen<
    F: Fn(Delivery) -> Pin<Box<(dyn Future<Output = ()> + 'static)>> + std::marker::Sync,
>(
    rmq_con: Connection,
    queue_definition: &Iter<'_, QueueDefinition<F>>,
) -> Result<()> {
    let channel: Channel = rmq_con.create_channel().await?;
    //let tmp = queue_definition.clone();
    for QueueDefinition {
        name,
        options,
        args,
        func,
    } in queue_definition.clone()
    {
        channel
            .queue_declare(&name, *options, args.clone())
            .await?;
        println!("rmq consumer connected, waiting for messages");
        consume_messages(channel.clone(), &"hello", "my_consumer", func.deref()).await;
    }
  //  let func_tmp = println!("rmq consumer connected, waiting for messages");
    //let a = consume_messages(channel.clone(), &"hello", "my_consumer", &tmp[0]);
    //let b = consume_messages(channel.clone(), "hello22", "my_consumer1");
/*   let consumers = vec![
        task::spawn(a), // Spawn a task for consumer1
                        //task::spawn(b), // Spawn a task for consumer2
    ];*/

   /* for consumer in test {
        println!("ici");
        consumer.await.expect("Erreur dans le consommateur");
    }*/
    Ok(())
}

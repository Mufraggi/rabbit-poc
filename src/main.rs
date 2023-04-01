use std::thread;
use std::time::Duration;
use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use deadpool_lapin::{Manager, Pool, PoolError};
use lapin::ConnectionProperties;
use lapin::options::{BasicAckOptions, BasicConsumeOptions, QueueDeclareOptions};
use lapin::types::FieldTable;
use thiserror::Error as ThisError;
use serde::Deserialize;
use serde::Serialize;
use tokio_amqp::LapinTokioExt;
use std::result::Result as StdResult;
use futures::{join, StreamExt};

type RMQResult<T> = StdResult<T, PoolError>;
type Result<T> = StdResult<T, Error>;
type Connection = deadpool::managed::Object<Manager>;


#[derive(ThisError, Debug)]
enum Error {
    #[error("rmq error: {0}")]
    RMQError(#[from] lapin::Error),
    #[error("rmq pool error: {0}")]
    RMQPoolError(#[from] PoolError),
}

#[derive(Serialize, PartialEq, Debug, Deserialize)]
struct Response {
    status: String,
}

pub async fn health() -> impl Responder {
    HttpResponse::Ok().content_type("application/json").body(
        serde_json::to_string(&Response {
            status: "OK".parse().unwrap(),
        })
            .unwrap(),
    )
}


#[tokio::main]
async fn main() -> Result<()> {
    let addr =
        std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://rmq:rmq@127.0.0.1:5672/%2f".into());
    let manager = Manager::new(addr, ConnectionProperties::default().with_tokio());
    let pool: Pool = deadpool::managed::Pool::builder(manager)
        .max_size(10)
        .build()
        .expect("can create pool");


    let actix_thread =
        HttpServer::new(move || {
            App::new()
                .route("/health", web::get().to(health))
        })
            .bind("127.0.0.1:8080").unwrap()
            .run();

    let rmq_thread =
        rmq_listen(pool.clone());


    println!("Started server at localhost:8000");
    let _ = join!(
        actix_thread,
    rmq_thread
    );
    Ok(())
}

async fn rmq_listen(pool: Pool) -> Result<()> {
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
                println!("received msg: {:?}",  String::from_utf8_lossy(&delivery.data));
                channel
                    .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
                    .await?
            }
            Err(_) => { (()) }
        }
    };
    Ok(())
}
mod api;
mod rabbit;
mod repository;

use crate::api::serve;
use crate::rabbit::rabbit_builder::{QueueDefinition, RabbitHandlerBuilder};
use futures::join;
use lapin::options::QueueDeclareOptions;
use lapin::types::FieldTable;
//use crate::rabbit::{RabbitHandler, rmq_listen};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let actix_thread = serve("localhost");
    let rabbit_handler = RabbitHandlerBuilder::new()
        .addr(
            std::env::var("AMQP_ADDR")
                .unwrap_or_else(|_| "amqp://rmq:rmq@127.0.0.1:5672/%2f".into()),
        )
        .push_queue_definition(QueueDefinition{
            name: "hello".to_string(),
            options: QueueDeclareOptions::default(),
            args: FieldTable::default(),
        })
        .push_queue_definition(QueueDefinition{
            name: "hello22".to_string(),
            options: QueueDeclareOptions::default(),
            args: FieldTable::default(),
        })
        .build()
        .unwrap();
    let rmq_thread = rabbit_handler.listen_rmq();
    let _ = join!(
     actix_thread,
    rmq_thread
    );
    Ok(())
}

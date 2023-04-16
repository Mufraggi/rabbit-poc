use std::future::Future;
use std::pin::Pin;
use crate::rabbit::rabbit_handler::RabbitHandler;
use deadpool_lapin::{Manager, Pool};
use lapin::options::QueueDeclareOptions;
use lapin::types::FieldTable;
use lapin::ConnectionProperties;
use lapin::message::Delivery;
use serde::Deserialize;
use serde::Serialize;
use tokio_amqp::LapinTokioExt;
use crate::rabbit::generic_function::FuncGeneric;

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub enum RabbitBuilderError {
    AddrNotSet,
    QueueDefinitionEmpty,
}

pub struct RabbitHandlerBuilder<F: Fn(Delivery) -> Pin<Box<(dyn Future<Output=()> + 'static)>> + 'static> {
    addr_set: bool,
    addr: String,
    queue_definitions: Vec<QueueDefinition<F>>,
    queue_definitions_set: bool,
}

pub struct QueueDefinition<F> where
    F: Fn(Delivery) -> Pin<Box<dyn Future<Output=()>>> + 'static, {
    pub name: String,
    pub options: QueueDeclareOptions,
    pub args: FieldTable,
    pub func: FuncGeneric<F>,
}



impl<F: Fn(Delivery) -> Pin<Box<(dyn Future<Output=()> + 'static)>>> RabbitHandlerBuilder<F> {
    pub fn new() -> RabbitHandlerBuilder<F> {
        RabbitHandlerBuilder {
            addr_set: false,
            addr: "".to_string(),
            queue_definitions: vec![],
            queue_definitions_set: false,
        }
    }

    pub fn addr(mut self, addr: String) -> RabbitHandlerBuilder<F> {
        self.addr = addr.to_string();
        self.addr_set = true;
        self
    }
    pub fn push_queue_definition(mut self, queue: QueueDefinition<F>) -> RabbitHandlerBuilder<F> {
        self.queue_definitions.push(queue);
        self.queue_definitions_set = true;
        self
    }
    pub fn build(self) -> Result<RabbitHandler<F>, RabbitBuilderError> {
        if !self.addr_set {
            return Err(RabbitBuilderError::AddrNotSet);
        }
        if !self.queue_definitions_set {
            return Err(RabbitBuilderError::QueueDefinitionEmpty);
        }
        let manager = Manager::new(self.addr, ConnectionProperties::default().with_tokio());
        let pool: Pool = Pool::builder(manager)
            .max_size(10)
            .build()
            .expect("can create pool");
        Ok(RabbitHandler {
            rabbit_pool: pool,
            queue_definition: self.queue_definitions,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;
    use lapin::message::Delivery;
    use crate::rabbit::rabbit_builder::{
        QueueDefinition, RabbitBuilderError, RabbitHandlerBuilder,
    };
    use lapin::options::QueueDeclareOptions;
    use lapin::types::FieldTable;
    use crate::rabbit::generic_function::FuncGeneric;

    /* #[test]
     fn test_builder_fail_no_addr() {
         let res = RabbitHandlerBuilder::new().build();
         assert!(res.is_err());
         assert_eq!(res.unwrap_err(), RabbitBuilderError::AddrNotSet)
     }
     #[test]
     fn test_builder_fail_no_queue_def() {
         let res = RabbitHandlerBuilder::new().addr("qq".parse().unwrap()).build();
         assert!(res.is_err());
         assert_eq!(res.unwrap_err(), RabbitBuilderError::QueueDefinitionEmpty)
     }*/
    #[tokio::test]
    async fn test_builder_work() {
        async fn add(a: u32, b: Delivery) -> () {
            println!("{:?}", b);
        }

        let add5 = move |x| Box::pin( async move { add(5, x) });
        let res = RabbitHandlerBuilder::new()
            .addr("qq".parse().unwrap())
            .push_queue_definition(QueueDefinition {
                name: "teest".to_string(),
                options: QueueDeclareOptions::default(),
                args: FieldTable::default(),
                func: FuncGeneric::new(move |b| {
                    Box::pin(async move {
                        add(5, b).await
                    })}),
            })
            .build();
        assert!(res.is_ok());
        let handler = res.unwrap();
        assert_eq!(handler.queue_definition.len(), 1);
        assert_eq!(handler.rabbit_pool.status().max_size, 10);
    }
}

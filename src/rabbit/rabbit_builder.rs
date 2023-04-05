use crate::rabbit::rabbit_handler::RabbitHandler;
use deadpool_lapin::{Manager, Pool};
use lapin::options::QueueDeclareOptions;
use lapin::types::FieldTable;
use lapin::ConnectionProperties;
use serde::Deserialize;
use serde::Serialize;
use tokio_amqp::LapinTokioExt;

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub enum RabbitBuilderError {
    AddrNotSet,
    QueueDefinitionEmpty,
}

pub struct RabbitHandlerBuilder {
    addr_set: bool,
    addr: String,
    queue_definitions: Vec<QueueDefinition>,
    queue_definitions_set: bool,
}
#[derive(Debug)]
pub struct QueueDefinition {
    pub name: String,
    pub options: QueueDeclareOptions,
    pub args: FieldTable,
}

 impl RabbitHandlerBuilder {
     pub fn new() -> RabbitHandlerBuilder {
        RabbitHandlerBuilder {
            addr_set: false,
            addr: "".to_string(),
            queue_definitions: vec![],
            queue_definitions_set: false,
        }
    }

     pub fn addr(mut self, addr: String) -> RabbitHandlerBuilder {
        self.addr = addr.to_string();
        self.addr_set = true;
        self
    }
     pub fn push_queue_definition(mut self, queue: QueueDefinition) -> RabbitHandlerBuilder {
        self.queue_definitions.push(queue);
        self.queue_definitions_set = true;
        self
    }
     pub fn build(self) -> Result<RabbitHandler, RabbitBuilderError> {
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
    use crate::rabbit::rabbit_builder::{
        QueueDefinition, RabbitBuilderError, RabbitHandlerBuilder,
    };
    use lapin::options::QueueDeclareOptions;
    use lapin::types::FieldTable;

    #[test]
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
    }
    #[tokio::test]
    async fn test_builder_work() {
        let res = RabbitHandlerBuilder::new()
            .addr("qq".parse().unwrap())
            .push_queue_definition(QueueDefinition {
                name: "teest".to_string(),
                options: QueueDeclareOptions::default(),
                args: FieldTable::default(),
            })
            .build();
        assert!(res.is_ok());
        let handler = res.unwrap();
        assert_eq!(handler.queue_definition.len(), 1);
        assert_eq!(handler.rabbit_pool.status().max_size, 10);
    }
}

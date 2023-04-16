use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use lapin::message::Delivery;

pub struct FuncGeneric<F>
    where
        F: Fn(Delivery) -> Pin<Box<dyn Future<Output = ()>>> + 'static ,
{
    func: F,
}

impl<F> FuncGeneric<F>
    where
        F: Fn(Delivery) -> Pin<Box<dyn Future<Output = ()>>> + 'static,
{
   pub fn new(func: F) -> Self {
        FuncGeneric { func }
    }

   pub async fn call(&self, b: Delivery) -> () {
        (self.func)(b).await
    }
}


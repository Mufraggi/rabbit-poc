/*
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use lapin::message::Delivery;

async fn add(a: u32, b: u32) -> u32 {
    a + b
}

struct MyStruct<F>
    where
        F: Fn(Delivery) -> Pin<Box<dyn Future<Output = ()>>> + 'static,
{
    func: F,
}

impl<F> MyStruct<F>
    where
        F: Fn(Delivery) -> Pin<Box<dyn Future<Output = ()>>> + 'static,
{
    fn new(func: F) -> Self {
        MyStruct { func }
    }

    async fn call(&self, b: u32) -> u32 {
        (self.func)(b).await
    }
}
*/
use async_trait::async_trait;
use std::fmt::Debug;

#[async_trait]
pub trait Source<Event: Send + 'static>: Sized {
    type Error: Debug + Send;

    async fn init() -> Result<Self, Self::Error>;
    async fn next(&mut self) -> Result<Event, Self::Error>;
}

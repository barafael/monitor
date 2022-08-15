use async_trait::async_trait;

#[async_trait]
pub trait Source<Event: Send + 'static>: Sized {
    type Error: std::fmt::Debug + Send;

    async fn init() -> Result<Self, Self::Error>;
    async fn next(&mut self) -> Result<Event, Self::Error>;
}

use async_trait::async_trait;

#[async_trait]
pub trait Source<E: Send + 'static>: Sized {
    type Error: std::fmt::Debug + Send;

    async fn init() -> Result<Self, Self::Error>;
    async fn next(&mut self) -> Result<E, Self::Error>;
}

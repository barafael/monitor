use std::time::Duration;

use async_trait::async_trait;
use futures_util::StreamExt;
use monitor::{Monitor, Source};
use tokio::{net::TcpStream, sync::broadcast};
use tokio_util::codec::{BytesCodec, FramedRead};

struct Client {
    reader: FramedRead<TcpStream, BytesCodec>,
}

#[async_trait]
impl Source<String> for Client {
    type Error = anyhow::Error;

    async fn init() -> Result<Self, Self::Error> {
        let stream = TcpStream::connect("localhost:8080").await?;
        let reader = FramedRead::new(stream, BytesCodec::new());
        Ok(Self { reader })
    }

    async fn next(&mut self) -> Result<String, Self::Error> {
        if let Some(Ok(msg)) = self.reader.next().await {
            Ok(format!("{msg:?}"))
        } else {
            Err(anyhow::Error::msg("drained"))
        }
    }
}

#[derive(Debug, Clone)]
struct ExpBackoff(Duration);
impl Iterator for ExpBackoff {
    type Item = Duration;
    fn next(&mut self) -> Option<Self::Item> {
        self.0 *= 2;
        Some(self.0)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    let (sender, mut receiver) = broadcast::channel(16);
    tokio::join!(
        async move {
            while let Ok(msg) = receiver.recv().await {
                println!("{msg}");
            }
        },
        Client::forever(sender, ExpBackoff(Duration::from_millis(10)))
    );
    Ok(())
}

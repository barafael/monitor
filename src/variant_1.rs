use std::time::Duration;

use async_trait::async_trait;
use tokio::{sync::broadcast, time::sleep};
use tokio_util::sync::CancellationToken;

pub use crate::source::Source;

impl<T, E: Send + 'static> Monitor<E> for T where T: Source<E> {}

#[async_trait]
pub trait Monitor<Event: Send + 'static>: Source<Event> {
    async fn until_cancel(
        sender: broadcast::Sender<Event>,
        token: CancellationToken,
    ) -> Result<(), Self::Error> {
        tokio::select! {
            _ = token.cancelled() => {
                Ok(())
            },
            _ = Self::forever(sender) => {
                Ok(())
            }
        }
    }

    async fn forever(sender: tokio::sync::broadcast::Sender<Event>) -> std::convert::Infallible
    where
        Self::Error: Send,
    {
        'init: loop {
            let mut instance = match Self::init().await {
                Ok(i) => i,
                Err(_e) => {
                    sleep(Duration::from_millis(100)).await;
                    continue 'init;
                }
            };

            'next: loop {
                let err = match instance.next().await {
                    Ok(data) => {
                        drop(sender.send(data));
                        continue 'next;
                    }
                    Err(e) => e,
                };
                log::warn!("Failed to receive next event: {err:?}");
                drop(instance);
                sleep(Duration::from_millis(100)).await;
                continue 'init;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use tokio::{sync::broadcast, time::Instant};
    use tokio_util::sync::CancellationToken;

    use crate::{test::Sensor, variant_1::Monitor};

    #[tokio::test(start_paused = true)]
    async fn restarts_on_error() {
        let token = CancellationToken::new();
        let (tx, mut rx) = broadcast::channel::<u8>(16);

        let hdl = tokio::spawn(Sensor::until_cancel(tx, token.clone()));
        let wait = tokio::spawn(async move {
            assert_eq!(1, rx.recv().await.unwrap());
            assert_eq!(2, rx.recv().await.unwrap());
            assert_eq!(3, rx.recv().await.unwrap());
            assert_eq!(4, rx.recv().await.unwrap());

            for _ in 0..4 {
                // poll until error
                let start = Instant::now();
                assert_eq!(rx.recv().await.unwrap(), 1);
                assert!(start.elapsed() > Duration::from_millis(95));
                assert!(start.elapsed() < Duration::from_millis(105));
                for i in [2, 3, 4] {
                    let v = rx.recv().await.unwrap();
                    assert_eq!(i, v);
                }
            }
            token.cancel();
        });
        let (hdl, _wait) = tokio::try_join!(hdl, wait).unwrap();
        assert!(hdl.is_ok());
    }
}

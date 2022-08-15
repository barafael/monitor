use std::time::Duration;

use async_trait::async_trait;
use status::Status;
use tokio::{sync::broadcast, time::sleep};
use tokio_util::sync::CancellationToken;

use self::repeat_last::LastRepeatIter;

mod repeat_last;
mod source;
mod status;
#[cfg(test)]
mod test;

mod variant_1;
mod variant_2;

pub use crate::source::Source;

impl<T, E: Send + 'static> Monitor<E> for T where T: Source<E> {}

#[async_trait]
pub trait Monitor<E: Send + 'static>: Source<E> {
    async fn until_cancel(
        sender: broadcast::Sender<E>,
        token: CancellationToken,
        backoff: impl Iterator<Item = Duration> + Send + Clone,
    ) -> Result<(), Self::Error> {
        tokio::select! {
            _ = token.cancelled() => {
                Ok(())
            },
            _ = Self::forever(sender, backoff) => {
                Ok(())
            }
        }
    }

    async fn forever(
        sender: tokio::sync::broadcast::Sender<E>,
        backoff_duration: impl Iterator<Item = Duration> + Send + Clone,
    ) -> std::convert::Infallible
    where
        Self::Error: Send,
    {
        let mut backoff = LastRepeatIter::new(backoff_duration.clone());
        let mut status = Status::default();

        'init: loop {
            let mut instance = match Self::init().await {
                Ok(i) => i,
                Err(e) => {
                    status.down(&format!("Failed to init: {e:?}"));
                    sleep(backoff.next().unwrap()).await;
                    continue 'init;
                }
            };

            status.up();
            backoff = LastRepeatIter::new(backoff_duration.clone());

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

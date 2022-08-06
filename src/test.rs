#[cfg(test)]
mod test {
    use std::{
        sync::atomic::{AtomicUsize, Ordering},
        time::Duration,
    };

    use async_trait::async_trait;
    use tokio::{sync::broadcast, time::Instant};
    use tokio_util::sync::CancellationToken;

    use crate::{Monitor, Source};

    struct Sensor(u8);

    #[async_trait]
    impl Source<u8> for Sensor {
        type Error = anyhow::Error;

        async fn init() -> Result<Self, Self::Error> {
            Ok(Self(0))
        }

        async fn next(&mut self) -> Result<u8, Self::Error> {
            self.0 += 1;
            if self.0 % 5 == 0 {
                anyhow::bail!("bah");
            }
            Ok(self.0)
        }
    }

    #[tokio::test(start_paused = true)]
    async fn restarts_on_error() {
        let iterator = std::iter::repeat(Duration::from_millis(500));
        let token = CancellationToken::new();
        let (tx, mut rx) = broadcast::channel::<u8>(16);

        let hdl = tokio::spawn(Sensor::until_cancel(tx, token.clone(), iterator));
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

    #[tokio::test(start_paused = true)]
    async fn backs_off() {
        #[derive(Debug, Clone)]
        struct ExpBackoff(Duration);
        impl Iterator for ExpBackoff {
            type Item = Duration;
            fn next(&mut self) -> Option<Self::Item> {
                self.0 *= 2;
                Some(self.0)
            }
        }

        let backoff_reference = ExpBackoff(Duration::from_millis(10));
        static INIT_COUNTER: AtomicUsize = AtomicUsize::new(0);
        struct Sensor;
        #[async_trait]
        impl Source<()> for Sensor {
            type Error = anyhow::Error;
            async fn init() -> Result<Self, Self::Error> {
                INIT_COUNTER.fetch_add(1, Ordering::SeqCst);
                anyhow::bail!("nope")
            }

            async fn next(&mut self) -> Result<(), Self::Error> {
                unreachable!()
            }
        }

        let (tx, mut rx) = broadcast::channel(16);
        let token = CancellationToken::new();

        let hdl = tokio::spawn(Sensor::until_cancel(tx, token.clone(), backoff_reference));
        let ensure_no_recv = tokio::spawn(async move {
            if let Err(_e) = tokio::time::timeout(Duration::from_secs(50), rx.recv()).await {
                token.cancel();
                return;
            } else {
                unreachable!()
            }
        });
        let (hdl, _ensure) = tokio::try_join!(hdl, ensure_no_recv).unwrap();
        hdl.unwrap();
        assert_eq!(12, INIT_COUNTER.load(Ordering::SeqCst));
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use futures::TryStreamExt;
    use pulsar::{ProducerOptions, TokioExecutor};

    async fn run(
        topic: &str,
        product_batch_size: u32,
        consumer_batch_size: u32,
    ) -> Result<(), pulsar::Error> {
        let pulsar = pulsar::Pulsar::builder("pulsar://127.0.0.1:6650", TokioExecutor)
            .build()
            .await?;

        // Consumer
        let mut consumer = pulsar
            .consumer()
            .with_topic(topic)
            .with_batch_size(consumer_batch_size)
            .build::<String>()
            .await?;
        tokio::spawn(async move {
            while let Ok(Some(m)) = consumer.try_next().await {
                let _ = consumer.ack(&m).await;
            }
            let _ = consumer.close().await;
        });

        // Producer
        let mut producer = pulsar
            .producer()
            .with_topic(topic)
            .with_options(ProducerOptions {
                batch_size: Some(product_batch_size),
                batch_byte_size: Some(128 * 1024),
                ..Default::default()
            })
            .build()
            .await?;
        for _ in 0..100000 {
            let _ = producer.send(format!("Message for {}", topic)).await;
        }
        drop(producer);

        // Wait for a while
        tokio::time::sleep(Duration::from_secs(3)).await;

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn it_works() -> Result<(), pulsar::Error> {
        run("good", 100, 1000).await
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn boom() -> Result<(), pulsar::Error> {
        run("bad", 1000, 100).await
    }
}

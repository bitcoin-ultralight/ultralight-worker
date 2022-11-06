use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::{types::ByteStream, Client};
use tokio::io::AsyncReadExt;

pub struct S3Pusher {
    client: Client,
    prefix: String,
}

impl S3Pusher {
    pub async fn new(prefix: String) -> anyhow::Result<Self> {
        let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
        let config = aws_config::from_env().region(region_provider).load().await;
        let client = Client::new(&config);

        Ok(Self { prefix, client })
    }

    pub async fn push_bytes(&self, key: &str, bytes: Vec<u8>) -> anyhow::Result<()> {
        self.client
            .put_object()
            .bucket("patbatch-outputs")
            .key(format!("{}{}", self.prefix, key))
            .body(ByteStream::from(bytes))
            .send()
            .await?;

        Ok(())
    }

    pub async fn pull_bytes(&self, key: &str) -> anyhow::Result<Vec<u8>> {
        let mut buf: Vec<u8> = Vec::new();
        self.client
            .get_object()
            .bucket("patbatch-outputs")
            .key(format!("{}{}", self.prefix, key))
            .send()
            .await
            .unwrap()
            .body
            .into_async_read()
            .read_to_end(&mut buf)
            .await?;
        return Ok(buf);
    }
}

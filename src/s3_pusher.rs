use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::{types::ByteStream, Client};

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
}

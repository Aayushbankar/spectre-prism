use reqwest::Client;
use prism_common::OcsfNetworkActivity;
use anyhow::Result;

pub struct HttpSink {
    client: Client,
    endpoint: String,
}

impl HttpSink {
    pub fn new(endpoint: &str) -> Self {
        Self {
            client: Client::new(),
            endpoint: endpoint.to_string(),
        }
    }

    pub async fn push_bulk(&self, batch: &[OcsfNetworkActivity]) -> Result<()> {
        let resp = self.client.post(&self.endpoint)
            .json(batch)
            .send()
            .await?;
        
        if !resp.status().is_success() {
            anyhow::bail!("Sink rejected bulk push: {}", resp.status());
        }
        
        Ok(())
    }
}

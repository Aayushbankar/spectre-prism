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
        let mut ndjson = String::new();
        for item in batch {
            ndjson.push_str("{\"index\":{}}\n");
            ndjson.push_str(&serde_json::to_string(item).unwrap());
            ndjson.push('\n');
        }

        let resp = self.client.post(&self.endpoint)
            .header("Content-Type", "application/x-ndjson")
            .body(ndjson)
            .send()
            .await?;
        
        let status = resp.status();
        let body = resp.text().await?;

        if !status.is_success() {
            anyhow::bail!("Sink rejected bulk push: {} - {}", status, body);
        }

        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
            if json.get("errors").and_then(|v| v.as_bool()).unwrap_or(false) {
                anyhow::bail!("Bulk push contained errors: {}", body);
            }
        } else {
            // fallback if it's not valid json but contains errors:true
            if body.contains("\"errors\":true") || body.contains("\"errors\": true") {
                anyhow::bail!("Bulk push contained errors: {}", body);
            }
        }
        
        Ok(())
    }
}

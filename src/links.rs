use std::str::FromStr;
use std::sync::Arc;
use futures_util::StreamExt;
use reqwest::header::{HeaderName, HeaderValue};
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use tokio_websockets::ClientBuilder;
use crate::{ok_or_return, ok_or_skip};

const ALL_DOMAINS_ENDPOINT: &str = "https://phish.sinking.yachts/v2/all";
const DOMAINS_FEED_ENDPOINT: &str = "wss://phish.sinking.yachts/feed";
const GITHUB_REPO_URL: &str = "https://github.com/banocean/custom";

#[derive(Clone)]
pub struct ScamLinks {
    discord_scam_domains: Arc<Mutex<Vec<String>>>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct UpdateMessage {
    #[serde(rename = "type")]
    action: String,
    domains: Vec<String>
}

impl ScamLinks {
    pub async fn new() -> Result<Self, String> {
        let domains = reqwest::get(ALL_DOMAINS_ENDPOINT).await.map_err(|err| format!("{err}"))?;
        let domains: Vec<String> = serde_json::from_str(domains.text().await.map_err(|err| format!("{err}"))?.as_str()).map_err(|err| format!("{err}"))?;

        Ok(Self {
            discord_scam_domains: Arc::new(Mutex::new(domains))
        })
    }

    pub fn connect(&self) {
        let discord_scam_domains = self.discord_scam_domains.clone();
        tokio::spawn(async move {
            let (mut client, _) = ClientBuilder::new()
                .uri(DOMAINS_FEED_ENDPOINT)
                .expect("Cannot parse url")
                .add_header(
                    HeaderName::from_str("X-Identity").unwrap(),
                    HeaderValue::from_str(GITHUB_REPO_URL).unwrap()
                )
                .expect("Cannot parse str to HeaderValue")
                .connect()
                .await
                .unwrap();

            while let Some(Ok(msg)) = client.next().await {
                let payload = ok_or_skip!(msg.as_text(), Some);
                let request = ok_or_return!(serde_json::from_str::<UpdateMessage>(payload), Ok);
                if request.action == *"add" {
                    for domain in request.domains {
                        discord_scam_domains.lock().await.push(domain);
                    }
                }
            };
        });
    }

    pub async fn contains(&self, domains: Vec<String>) -> bool {
        let scam_domains = self.discord_scam_domains.lock().await;
        for domain in domains {
            if scam_domains.contains(&domain) { return true }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use crate::links::ScamLinks;

    #[tokio::test]
    async fn test_contains_method() {
        let scam_links = ScamLinks::new().await.unwrap();
        assert_eq!(
            scam_links.contains(vec!["moderating-verified-school.club".to_string()]).await,
            true
        );
        assert_eq!(
            scam_links.contains(vec!["".to_string()]).await,
            false
        );
    }
}
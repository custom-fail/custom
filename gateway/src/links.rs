use std::sync::Arc;
use tokio::sync::Mutex;

const ALL_DOMAINS_ENDPOINT: &str = "https://phish.sinking.yachts/v2/all";

pub struct ScamLinks {
    discord_scam_domains: Arc<Mutex<Vec<String>>>
}

impl ScamLinks {
    pub async fn new() -> Result<Self, String> {

        let links = reqwest::get(ALL_DOMAINS_ENDPOINT).await.map_err(|err| format!("{err}"))?;
        let links: Vec<String> = serde_json::from_str(links.text().await.map_err(|err| format!("{err}"))?.as_str()).map_err(|err| format!("{err}"))?;

        Ok(Self {
            discord_scam_domains: Arc::new(Mutex::new(links))
        })

    }

    pub async fn contains(&self, domains: Vec<String>) -> bool {

        let scam_domains = self.discord_scam_domains.lock().await;

        for domain in domains {
            if scam_domains.contains(&domain) {
                return false
            }
        }

        false

    }
}
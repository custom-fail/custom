use tokio::fs;

use super::Asset;

pub async fn load(path: Option<String>) -> Asset {
    let path = path.unwrap_or("messages.toml".to_string());
    let default_asset = fs::read_to_string(path.to_owned())
        .await.expect(format!("Cannot load {path}").as_str());

    let default_asset: Asset = toml::from_str(&default_asset)
        .expect(format!("Cannot parse {path}").as_str());
    
    default_asset
}
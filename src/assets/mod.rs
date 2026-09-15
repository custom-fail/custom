use std::{collections::HashMap, sync::Arc};

use crate::utils::cli;
use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::sync::RwLock;

use self::message::Message;

mod message;
pub mod render;

pub type Asset = HashMap<String, Arc<Message>>;

pub struct AssetsManager {
    pub default: Asset,
    pub custom: RwLock<HashMap<String, Arc<Asset>>>,
}

const URL: &str = "https://raw.githubusercontent.com/oceaann/custom/dev/messages.yaml";
async fn fetch_default_asset() -> Option<String> {
    match reqwest::get(URL).await {
        Ok(res) => res.text().await.ok(),
        Err(err) => {
            eprintln!("Cannot fetch assets - applying defaults (empty values): {err}");
            None
        }
    }
}

fn parse_asset_config(asset: &str) -> Result<Asset, toml::de::Error> {
    toml::from_str(asset)
}

const ASSETS_CONFIG_PATH: &str = "./messages.toml";
const CANNOT_PARSE_FETCHED_CONFIG: &str = "Cannot parse auto-downloaded message configuration";

async fn load_default(path: Option<String>) -> Asset {
    let path = path.unwrap_or(ASSETS_CONFIG_PATH.to_string());
    let file = fs::read_to_string(path.to_owned()).await;

    match file {
        Ok(file) => {
            parse_asset_config(file.as_str()).expect(format!("Cannot parse {path}").as_str())
        }
        Err(error) => {
            if error.kind() == std::io::ErrorKind::NotFound {
                if cli::confirm(
                    "Cannot load file {path} do you want to fetch the recommended configuration",
                ) {
                    println!("Fetching {URL}");
                    fetch_default_asset()
                        .await
                        .map(|asset| {
                            parse_asset_config(asset.as_str()).expect(CANNOT_PARSE_FETCHED_CONFIG)
                        })
                        .unwrap_or_default()
                } else {
                    Default::default()
                }
            } else {
                eprintln!(
                    "Cannot load assets with default messages: {}",
                    error.to_string()
                );
                Default::default()
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GuildAssets(pub Vec<String>);

impl AssetsManager {
    pub async fn new() -> Self {
        Self {
            default: load_default(None).await,
            custom: Default::default(),
        }
    }
}

#[macro_export]
macro_rules! render_context {
    ($([$name: expr, $value: expr]),* ) => {
        {
            let mut ctx = tera::Context::new();
            $(
                ctx.insert($name, $value);
            )*
            ctx
        }
    };
}

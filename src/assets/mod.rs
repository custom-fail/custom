use std::{sync::Arc, collections::HashMap};

use tokio::sync::RwLock;

use self::message::Message;

mod load;
mod message;
mod render;

pub type Asset = HashMap<String, Message>;

pub struct AssetsManager {
    pub default: Asset,
    pub custom: RwLock<HashMap<String, Arc<Asset>>>
}

pub struct GuildAssets(Vec<String>);

impl AssetsManager {
    pub async fn new() -> Self {
        Self { default: load::load(None).await, custom: Default::default() }
    }
}

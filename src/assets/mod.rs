use std::{sync::Arc, collections::HashMap};

use serde::{Serialize, Deserialize};
use tokio::sync::RwLock;

use self::message::Message;

mod load;
mod message;
pub mod render;

pub type Asset = HashMap<String, Message>;

pub struct AssetsManager {
    pub default: Asset,
    pub custom: RwLock<HashMap<String, Arc<Asset>>>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GuildAssets(pub Vec<String>);

impl AssetsManager {
    pub async fn new() -> Self {
        Self { default: self::load::load(None).await, custom: Default::default() }
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

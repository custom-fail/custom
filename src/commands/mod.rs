pub mod top;
pub mod case;
pub mod context;
pub mod moderation;
pub mod options;
pub mod settings;

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use twilight_http::Client;
use twilight_model::http::interaction::{InteractionResponseData, InteractionResponseType};
use crate::commands::context::InteractionContext;
use crate::{MongoDBConnection, RedisConnection};
use crate::models::config::GuildConfig;
use crate::utils::errors::Error;

pub type ResponseData = Result<(InteractionResponseData, Option<InteractionResponseType>), Error>;
pub type Response = Pin<Box<dyn Future<Output = ResponseData> + Send + 'static>>;
type Callback = fn(InteractionContext, MongoDBConnection, RedisConnection, Arc<Client>, GuildConfig) -> Response;

#[macro_export]
macro_rules! command {
    ($name: expr, $module: expr, $function: expr) => {
        Command {
            name: $name,
            module: $module,
            run: |interaction: InteractionContext, mongodb: MongoDBConnection, redis: RedisConnection, discord_http: Arc<Client>, config: GuildConfig| ($function)(interaction, mongodb, redis, discord_http, config).boxed()
        }
    }
}

#[derive(Clone)]
pub struct Command {
    pub name: String,
    pub module: String,
    pub run: Callback,
}
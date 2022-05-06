pub mod top;
pub mod case;
pub mod context;
pub mod moderation;
pub mod levels;

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use twilight_http::Client;
use twilight_model::application::interaction::application_command::CommandOptionValue::{SubCommand, SubCommandGroup};
use twilight_model::application::interaction::application_command::{CommandData, CommandOptionValue};
use twilight_model::http::interaction::{InteractionResponseData, InteractionResponseType};
use database::models::config::GuildConfig;
use database::mongodb::MongoDBConnection;
use database::redis::RedisConnection;
use utils::errors::Error;
use crate::commands::context::InteractionContext;

pub type ResponseData = Result<(InteractionResponseData, Option<InteractionResponseType>), Error>;
pub type Response = Pin<Box<dyn Future<Output = ResponseData> + Send + 'static>>;
type Callback = fn(InteractionContext, MongoDBConnection, RedisConnection, Arc<Client>, GuildConfig) -> Response;

#[macro_export]
macro_rules! command {
    ($name: expr, $module: expr, $function: expr) => {
        Command::new(
            $name,
            $module,
            |interaction: InteractionContext, mongodb: MongoDBConnection, redis: RedisConnection, discord_http: Arc<Client>, config: GuildConfig| ($function)(interaction, mongodb, redis, discord_http, config).boxed()
        )
    }
}

#[derive(Clone)]
pub struct Command {
    pub name: String,
    pub module: String,
    pub run: Callback,
}

impl Command {
    pub fn new(name: &str, module: &str, run: Callback) -> Self {
        Self {
            name: name.to_string(),
            module: module.to_string(),
            run,
        }
    }
}

pub fn parse_slash_command_to_text(slash_command_data: CommandData) -> Vec<String> {
    if slash_command_data.options.is_empty() {
        handle_value(
            vec![slash_command_data.name.to_lowercase()],
            slash_command_data.options[0].name.clone(),
            slash_command_data.options[0].value.to_owned(),
        )
    } else {
        vec![slash_command_data.name.to_lowercase()]
    }
}

fn handle_value(mut before: Vec<String>, name: String, slash_command_value: CommandOptionValue) -> Vec<String> {
    match slash_command_value {
        SubCommandGroup(value) => handle_value(
            { before.push(name.to_lowercase()); before },
            value[0].name.to_owned(),
            value[0].value.to_owned(),
        ),
        SubCommand(_) => { before.push(name.to_lowercase()); before },
        _ => before,
    }
}
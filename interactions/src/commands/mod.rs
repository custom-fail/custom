pub mod top;
pub mod case;
pub mod context;

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use twilight_http::Client;
use twilight_model::application::callback::CallbackData;
use twilight_model::application::interaction::application_command::CommandOptionValue::{SubCommand, SubCommandGroup};
use twilight_model::application::interaction::application_command::{CommandData, CommandDataOption, CommandOptionValue};
use twilight_model::application::interaction::ApplicationCommand;
use database::mongodb::MongoDBConnection;
use database::redis::RedisConnection;
use crate::commands::context::CommandContext;

pub type Response = Pin<Box<dyn Future<Output = Result<CallbackData, String>> + Send + 'static>>;
type Callback = fn(CommandContext, MongoDBConnection, RedisConnection, Arc<Client>) -> Response;

macro_rules! command {
    ($name: expr, $module: expr, $function: expr) => {
        Command::new(
            $name,
            $module,
            |interaction: CommandContext, mongodb: MongoDBConnection, redis: RedisConnection, discord_http: Arc<Client>| ($function)(interaction, mongodb, redis, discord_http).boxed()
        )
    }
}

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

impl Clone for Command {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            module: self.module.clone(),
            run: self.run.clone(),
        }
    }
}

pub fn parse_slash_command_to_text(slash_command_data: CommandData) -> Vec<String> {
    if slash_command_data.options.len() != 0 {
        handle_value(
            vec![slash_command_data.name],
            slash_command_data.options[0].name.to_owned(),
            slash_command_data.options[0].value.to_owned(),
        )
    } else {
        vec![slash_command_data.name]
    }
}

fn handle_value(mut before: Vec<String>, name: String, slash_command_value: CommandOptionValue) -> Vec<String> {
    match slash_command_value {
        SubCommandGroup(value) => handle_value(
            { before.push(name); before },
            value[0].name.to_owned(),
            value[0].value.to_owned(),
        ),
        SubCommand(_) => { before.push(name); before },
        _ => before,
    }
}
use std::future::Future;
use twilight_model::application::callback::CallbackData;
use twilight_model::application::interaction::application_command::CommandOptionValue::{SubCommand, SubCommandGroup};
use twilight_model::application::interaction::application_command::{CommandData, CommandOptionValue};
use twilight_model::application::interaction::ApplicationCommand;
use database::mongodb::MongoDBConnection;
use database::redis::RedisConnection;

pub type CommandResult = Box<dyn Future<Output = Result<CallbackData, &'static str>> + Unpin>;
pub type Callback = fn(Box<ApplicationCommand>, MongoDBConnection, RedisConnection) -> CommandResult;

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
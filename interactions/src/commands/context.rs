use std::any::TypeId;
use std::collections::HashMap;
use twilight_model::application::interaction::application_command::{CommandData, CommandDataOption, CommandInteractionDataResolved, CommandOptionValue};
use twilight_model::application::interaction::application_command::CommandOptionValue::{SubCommand, SubCommandGroup};
use twilight_model::application::interaction::ApplicationCommand;
use twilight_model::guild::PartialMember;
use twilight_model::id::Id;
use twilight_model::id::marker::{GenericMarker, GuildMarker, UserMarker};
use twilight_model::user::User;

#[derive(Debug)]
pub struct CommandContext {
    pub options: HashMap<String, CommandOptionValue>,
    pub command_text: String,
    pub custom_id: Option<String>,
    pub member: Option<PartialMember>,
    pub user: Option<User>,
    pub resolved: CommandInteractionDataResolved,
    pub target_id: Option<Id<GenericMarker>>,
    pub guild_id: Option<Id<GuildMarker>>
}

impl CommandContext {
    pub fn from_command_data(command: Box<ApplicationCommand>, command_text: String) -> Self {
        let command_data = command.data.clone();
        let mut command_options = HashMap::new();
        for (name, value) in parse_options_to_value(command_data.options) {
            command_options.insert(name, value);
        }
        let user = match command.member.clone() {
            Some(value) => match value.user {
                Some(user) => Some(user),
                None => command.user
            }
            None => command.user
        };
        Self {
            options: command_options,
            command_text,
            custom_id: None,
            member: command.member.clone(),
            user,
            resolved: command.data.resolved.unwrap_or(CommandInteractionDataResolved {
                channels: HashMap::new(),
                members: HashMap::new(),
                messages: HashMap::new(),
                roles: HashMap::new(),
                users: HashMap::new()
            }),
            target_id: command.data.target_id,
            guild_id: command.guild_id
        }
    }
}

fn parse_options_to_value(command_data_options: Vec<CommandDataOption>) -> Vec<(String, CommandOptionValue)> {
    command_data_options.iter().map(|command_data_option| {
        match command_data_option.value.clone() {
            SubCommand(command) => {
                parse_options_to_value(command)
            }
            SubCommandGroup(command) => {
                parse_options_to_value(command)
            }
            _ => Vec::from([(command_data_option.name.clone(), command_data_option.value.clone())])
        }
    }).collect::<Vec<Vec<(String, CommandOptionValue)>>>().concat()
}
use std::collections::HashMap;
use std::str::FromStr;
use twilight_model::application::interaction::application_command::{CommandDataOption, CommandInteractionDataResolved, CommandOptionValue};
use twilight_model::application::interaction::application_command::CommandOptionValue::{SubCommand, SubCommandGroup};
use twilight_model::application::interaction::{ApplicationCommand, MessageComponentInteraction};
use twilight_model::guild::PartialMember;
use twilight_model::id::Id;
use twilight_model::id::marker::{GenericMarker, GuildMarker};
use twilight_model::user::User;
use crate::Application;

#[derive(Debug)]
pub struct InteractionContext {
    pub options: HashMap<String, CommandOptionValue>,
    pub command_vec: Vec<String>,
    pub command_text: String,
    pub custom_id: Option<String>,
    pub member: Option<PartialMember>,
    pub user: Option<User>,
    pub resolved: CommandInteractionDataResolved,
    pub target_id: Option<Id<GenericMarker>>,
    pub guild_id: Option<Id<GuildMarker>>
}

#[macro_export]
macro_rules! check_type {
    ($value: expr, $type: path) => {
        match $value {
            $type(v) => Some(v),
            _ => None
        }
    }
}

impl InteractionContext {
    pub fn from_command_data(command: Box<ApplicationCommand>, subcommands: (Vec<String>, String)) -> Self {

        let mut command_options = HashMap::new();

        for (name, value) in parse_options_to_value(command.data.options) {
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
            command_vec: subcommands.0,
            command_text: subcommands.1,
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

    pub async fn from_message_component_interaction(interaction: Box<MessageComponentInteraction>, application: Application) -> Result<Self, String> {

        let id_vec = interaction.data.custom_id.split(":").collect::<Vec<&str>>();
        let command_id = id_vec.get(2).ok_or("There is no command data".to_string())?.to_string();

        let application_component = application.find_component(command_id).await
            .ok_or("There is no component with matching id".to_string())?;

        let name = application_component.command;

        let mut options = HashMap::new();

        for i in 0..application_component.options.len() {
            let value = if let Some(value) = id_vec.get(i + 3) { value } else { break };
            let (key, kind) = application_component.options[i].clone();
            let value = convert_value_to_option(value.to_string(), kind)?;
            options.insert(key, value);
        }

        for i in 0..application_component.values.len() {
            let value = if let Some(value) = interaction.data.values.get(i) { value.clone() } else { break };
            let (key, kind) = application_component.values[i].clone();
            let value = convert_value_to_option(value.to_string(), kind)?;
            options.insert(key, value);
        }

        let user = match interaction.member.clone() {
            Some(value) => match value.user {
                Some(user) => Some(user),
                None => interaction.user
            }
            None => interaction.user
        };

        Ok(Self {
            options,
            command_vec: vec![name.clone()],
            command_text: name,
            custom_id: Some(interaction.data.custom_id),
            member: interaction.member,
            user,
            resolved: CommandInteractionDataResolved {
                channels: HashMap::new(),
                members: HashMap::new(),
                messages: HashMap::new(),
                roles: HashMap::new(),
                users: HashMap::new()
            },
            target_id: None,
            guild_id: interaction.guild_id
        })
    }
}

fn convert_value_to_option(value: String, kind: String) -> Result<CommandOptionValue, String> {
    Ok(if kind == "String".to_string() {
        CommandOptionValue::String(value)
    } else if kind == "Boolean".to_string() {
        CommandOptionValue::Boolean(
            if value == "true".to_string() { true } else { false }
        )
    } else if kind == "Integer" {
        CommandOptionValue::Integer(
            value.parse::<i64>().map_err(|_| "Invalid option type".to_string())?
        )
    } else if kind == "User" {
        CommandOptionValue::User(
            Id::from_str(value.as_str()).map_err(|_| "Invalid option type".to_string())?
        )
    } else { return Err("Invalid option type".to_string()) })
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
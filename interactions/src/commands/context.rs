use std::collections::HashMap;
use std::str::FromStr;
use twilight_model::application::interaction::application_command::{CommandDataOption, CommandInteractionDataResolved, CommandOptionValue};
use twilight_model::application::interaction::application_command::CommandOptionValue::{SubCommand, SubCommandGroup};
use twilight_model::application::interaction::{ApplicationCommand, MessageComponentInteraction};
use twilight_model::application::interaction::modal::ModalSubmitInteraction;
use twilight_model::guild::PartialMember;
use twilight_model::id::Id;
use twilight_model::id::marker::{ChannelMarker, GenericMarker, GuildMarker};
use twilight_model::user::User;
use utils::{ok_or_break, ok_or_skip};
use utils::errors::Error;
use crate::Application;

#[derive(Debug, Clone)]
pub struct InteractionContext {
    pub options: HashMap<String, CommandOptionValue>,
    pub command_vec: Vec<String>,
    pub command_text: String,
    pub custom_id: Option<String>,
    pub channel_id: Id<ChannelMarker>,
    pub member: Option<PartialMember>,
    pub user: Option<User>,
    pub resolved: CommandInteractionDataResolved,
    pub target_id: Option<Id<GenericMarker>>,
    pub guild_id: Option<Id<GuildMarker>>
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
            channel_id: command.channel_id,
            member: command.member.clone(),
            user,
            resolved: command.data.resolved.unwrap_or(CommandInteractionDataResolved {
                attachments: HashMap::new(),
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

    pub async fn from_message_component_interaction(interaction: Box<MessageComponentInteraction>, application: Application) -> Result<Self, Error> {

        let id_vec = interaction.data.custom_id.split(':').collect::<Vec<&str>>();
        let command_id = id_vec.get(2).ok_or("There is no command data")?.to_string();

        let application_component = application.find_component(command_id).await
            .ok_or("There is no component with matching id")?;

        let name = application_component.command;

        let mut options = HashMap::new();

        for i in 0..application_component.options.len() {
            let value = ok_or_break!(id_vec.get(i + 3), Some);
            let (key, kind) = application_component.options[i].clone();
            let value = convert_value_to_option(value.to_string(), kind)?;
            options.insert(key, value);
        }

        for i in 0..application_component.values.len() {
            let value = ok_or_break!(interaction.data.values.get(i), Some);
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
        }.ok_or("Cannot get information about executor")?;

        if !interaction.data.custom_id.starts_with(
            format!("a:{}", user.id).as_str()
        ) && !interaction.data.custom_id.starts_with("a:*") {
            return Err(Error::from("This place is not dedicated for you :eyes:"))
        }

        Ok(Self {
            options,
            command_vec: vec![name.clone()],
            command_text: name,
            custom_id: Some(interaction.data.custom_id),
            channel_id: interaction.channel_id,
            member: interaction.member,
            user: Some(user),
            resolved: CommandInteractionDataResolved {
                attachments: HashMap::new(),
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

    pub async fn from_modal_submit_interaction(interaction: Box<ModalSubmitInteraction>, application: Application) -> Result<Self, Error> {

        let id_vec = interaction.data.custom_id.split(':').collect::<Vec<&str>>();
        let name = id_vec.get(1).ok_or("Cannot extract command from custom_ids")?.to_string();

        let modal = application.find_modal(name).await.ok_or("Unknown modal")?;

        let mut options = HashMap::new();

        for i in 0..modal.options.len() {
            let value = ok_or_break!(id_vec.get(i + 2), Some);
            let (key, kind) = modal.options[i].clone();
            let value = convert_value_to_option(value.to_string(), kind)?;
            options.insert(key, value);
        }

        for action_row in interaction.data.components {
            for text_input in action_row.components {
                let kind = ok_or_skip!(modal.inputs.get(text_input.custom_id.as_str()), Some);
                let value = convert_value_to_option(text_input.value, kind)?;
                options.insert(text_input.custom_id, value);
            }
        }

        let user = match interaction.member.clone() {
            Some(value) => match value.user {
                Some(user) => Some(user),
                None => interaction.user
            }
            None => interaction.user
        }.ok_or("Cannot get information about executor")?;

        Ok(Self {
            options,
            command_vec: vec![modal.command.clone()],
            command_text: modal.command,
            custom_id: Some(interaction.data.custom_id),
            channel_id: interaction.channel_id,
            member: interaction.member,
            user: Some(user),
            resolved: CommandInteractionDataResolved {
                attachments: HashMap::new(),
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

fn convert_value_to_option(value: String, kind: String) -> Result<CommandOptionValue, Error> {
    Ok(if kind == "String" {
        CommandOptionValue::String(value)
    } else if kind == "Boolean" {
        CommandOptionValue::Boolean(value == "true")
    } else if kind == "Integer" {
        CommandOptionValue::Integer(
            value.parse::<i64>().map_err(|_| "Invalid option type".to_string())?
        )
    } else if kind == "User" {
        CommandOptionValue::User(
            Id::from_str(value.as_str()).map_err(|_| "Invalid option type".to_string())?
        )
    } else { return Err(Error::from("Invalid option type")) })
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
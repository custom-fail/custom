mod activity;
mod cases;
mod moderation;
pub mod bitfield;

use crate::models::config::GuildConfig;
use twilight_model::application::command::{
    Command, CommandOption, CommandOptionType, CommandType,
};
use twilight_model::application::interaction::InteractionContextType;
use twilight_model::guild::Permissions;
use twilight_model::id::Id;
use twilight_model::oauth::ApplicationIntegrationType;

pub(self) fn defaults_command() -> Command {
    Command {
        application_id: None,
        contexts: Some(vec![InteractionContextType::Guild]),
        default_member_permissions: Some(Permissions::empty()),
        #[allow(deprecated)]
        dm_permission: None,
        description: "".to_string(),
        description_localizations: None,
        guild_id: None,
        id: None,
        integration_types: Some(vec![ApplicationIntegrationType::GuildInstall]),
        kind: CommandType::ChatInput,
        name: "".to_string(),
        name_localizations: None,
        nsfw: None,
        options: vec![],
        version: Id::new(1),
    }
}

pub(self) fn default_option() -> CommandOption {
    CommandOption {
        autocomplete: None,
        channel_types: None,
        choices: None,
        description: "".to_string(),
        description_localizations: None,
        kind: CommandOptionType::SubCommand,
        max_length: None,
        max_value: None,
        min_length: None,
        min_value: None,
        name: "".to_string(),
        name_localizations: None,
        options: None,
        required: None,
    }
}

macro_rules! options {
    ($name: expr, $description: expr, $kind: expr, $required: expr) => {
        CommandOption {
            name: $name.to_string(),
            kind: $kind,
            required: Some($required),
            description: $description.to_string(),
            ..default_option()
        }
    };
}
pub(self) use options;

macro_rules! subcommand {
    ($name: expr, $description: expr, $options: expr) => {
        CommandOption {
            name: $name.to_string(),
            description: $description.to_string(),
            kind: CommandOptionType::SubCommand,
            options: Some($options),
            ..default_option()
        }
    };
}
pub(self) use subcommand;

macro_rules! define_context_menu {
    ($name: tt, $command_name: expr, $permissions: expr) => {
        fn $name() -> Command {
            twilight_model::application::command::Command {
                name: $command_name.to_string(),
                kind: twilight_model::application::command::CommandType::User,
                default_member_permissions: Some($permissions),
                ..defaults_command()
            }
        }
    };
}

pub(self) use define_context_menu;

macro_rules! define_command {
    ($name: tt, $command_name: expr, $description: expr, $options: expr, $permissions: expr) => {
        fn $name() -> Command {
            twilight_model::application::command::Command {
                name: $command_name.to_string(),
                description: $description.to_string(),
                options: $options,
                default_member_permissions: Some($permissions),
                ..defaults_command()
            }
        }
    };
}
pub(self) use define_command;

macro_rules! choice {
    ($name: expr) => {
        twilight_model::application::command::CommandOptionChoice {
            name: $name.to_string(),
            name_localizations: None,
            value: twilight_model::application::command::CommandOptionChoiceValue::String(
                $name.to_string(),
            ),
        }
    };
}
pub(self) use choice;

macro_rules! if_enabled {
    ($config: expr, $field: tt, $commands: expr, $value: expr) => {
        if $config.enabled_commands.$field {
            $commands.push($value);
        }
    };
}
pub(self) use if_enabled;

use crate::server::guild::commands::activity::add_activity_commands;
use crate::server::guild::commands::cases::add_case_command;
use crate::server::guild::commands::moderation::add_moderation_commands;

pub fn get_guild_commands_list(config: &GuildConfig) -> Vec<Command> {
    let mut commands = vec![];

    add_moderation_commands(config, &mut commands);
    add_activity_commands(config, &mut commands);
    add_case_command(config, &mut commands);

    commands
}

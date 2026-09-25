use crate::models::config::GuildConfig;
use crate::models::config::moderation::MuteMode;
use crate::server::guild::commands::{
    choice, default_option, defaults_command, define_command, define_context_menu, if_enabled,
    options,
};
use twilight_model::application::command::{
    Command, CommandOption, CommandOptionType, CommandOptionValue,
};
use twilight_model::guild::Permissions;

fn moderation_action() -> Vec<CommandOption> {
    vec![
        options!("member", "Punished user", CommandOptionType::User, true),
        options!(
            "reason",
            "Reason for punishment",
            CommandOptionType::String,
            false
        ),
    ]
}

fn moderation_action_with_duration(required: bool) -> Vec<CommandOption> {
    vec![
        options!("member", "Punished user", CommandOptionType::User, true),
        options!(
            "duration",
            "Duration of punishment (eg. 1 minute, 1 week, 10m, 30s)",
            CommandOptionType::String,
            required
        ),
        options!(
            "reason",
            "Reason for punishment",
            CommandOptionType::String,
            false
        ),
    ]
}

define_command!(
    timeout_command,
    "timeout",
    "Timeouts users",
    moderation_action_with_duration(true),
    Permissions::MODERATE_MEMBERS
);
define_command!(
    mute_command,
    "mute",
    "Mutes users",
    moderation_action_with_duration(false),
    Permissions::MODERATE_MEMBERS
);
define_command!(
    kick_command,
    "kick",
    "Kicks users",
    moderation_action(),
    Permissions::KICK_MEMBERS
);
define_command!(
    warn_command,
    "warn",
    "Warns users",
    moderation_action(),
    Permissions::MODERATE_MEMBERS
);
define_command!(
    ban_command,
    "ban",
    "Bans users",
    moderation_action_with_duration(false),
    Permissions::BAN_MEMBERS
);

define_context_menu!(
    timeout_context_menu,
    "Timeout",
    Permissions::MODERATE_MEMBERS
);
define_context_menu!(mute_context_menu, "Mute", Permissions::MODERATE_MEMBERS);
define_context_menu!(kick_context_menu, "Kick", Permissions::KICK_MEMBERS);
define_context_menu!(warn_context_menu, "Warn", Permissions::MODERATE_MEMBERS);
define_context_menu!(ban_context_menu, "Ban", Permissions::BAN_MEMBERS);

fn clear_options() -> Vec<CommandOption> {
    vec![
        CommandOption {
            name: "amount".to_string(),
            description: "Amount of messages you want to consider for deletion".to_string(),
            min_value: Some(CommandOptionValue::Integer(1)),
            max_value: Some(CommandOptionValue::Integer(600)),
            kind: CommandOptionType::Integer,
            required: Some(true),
            ..default_option()
        },
        options!("author", "Message author", CommandOptionType::User, false),
        CommandOption {
            name: "filter".to_string(),
            kind: CommandOptionType::String,
            description: "Message characteristic you want to filter by".to_string(),
            choices: Some(vec![
                choice!("system"),
                choice!("attachments"),
                choice!("stickers"),
                choice!("embeds"),
                choice!("bots"),
            ]),
            ..default_option()
        },
    ]
}

define_command!(
    clear_command,
    "clear",
    "Deletes given amount of messages",
    clear_options(),
    Permissions::MANAGE_MESSAGES
);

pub(super) fn add_moderation_commands(config: &GuildConfig, commands: &mut Vec<Command>) {
    if let Some(moderation) = &config.moderation {
        match moderation.mute_mode {
            MuteMode::DependOnCommand => {
                if_enabled!(config, timeout, commands, timeout_command());
                if_enabled!(config, mute, commands, mute_command());

                if moderation.context_menu {
                    if_enabled!(config, timeout, commands, timeout_context_menu());
                    if_enabled!(config, mute, commands, mute_context_menu());
                }
            }
            MuteMode::Timeout => {
                if_enabled!(config, timeout, commands, timeout_command());
                if moderation.context_menu {
                    if_enabled!(config, timeout, commands, timeout_context_menu());
                }
            }
            MuteMode::Role => {
                if_enabled!(config, mute, commands, mute_command());
                if moderation.context_menu {
                    if_enabled!(config, mute, commands, mute_context_menu());
                }
            }
        };

        if_enabled!(config, warn, commands, warn_command());
        if_enabled!(config, kick, commands, kick_command());
        if_enabled!(config, ban, commands, ban_command());

        if moderation.context_menu {
            if_enabled!(config, warn, commands, warn_context_menu());
            if_enabled!(config, kick, commands, kick_context_menu());
            if_enabled!(config, ban, commands, ban_context_menu());
        }

        if_enabled!(config, clear, commands, clear_command());
    }
}

use crate::models::config::GuildConfig;
use crate::server::guild::commands::{default_option, defaults_command, if_enabled, options};
use twilight_model::application::command::{Command, CommandOption, CommandOptionType};

macro_rules! subcommand {
    ($name: expr, $description: expr) => {
        CommandOption {
            name: $name.to_string(),
            description: $description.to_string(),
            kind: CommandOptionType::SubCommand,
            ..default_option()
        }
    };
}

macro_rules! group {
    ($name: expr, $subcommands: expr) => {
        CommandOption {
            name: $name.to_string(),
            description: "-".to_string(),
            kind: CommandOptionType::SubCommandGroup,
            options: Some($subcommands),
            ..default_option()
        }
    };
}

pub(super) fn add_activity_commands(config: &GuildConfig, commands: &mut Vec<Command>) {
    if let Some(top) = &config.top {
        let mut subcommand_groups = vec![];

        if top.day {
            let mut subcommands = vec![];

            if_enabled!(
                config,
                top_day_all,
                subcommands,
                subcommand!("all", "Shows top 3 active users")
            );

            if_enabled!(
                config,
                top_day_me,
                subcommands,
                subcommand!("me", "Shows your position on the leaderboard")
            );

            if !subcommands.is_empty() {
                subcommand_groups.push(group!("day", subcommands));
            }
        }

        if top.week {
            let mut subcommands = vec![];

            if_enabled!(
                config,
                top_week_all,
                subcommands,
                subcommand!("all", "Shows top 3 active users")
            );

            if_enabled!(
                config,
                top_week_me,
                subcommands,
                subcommand!("me", "Shows your position on the leaderboard")
            );

            if !subcommands.is_empty() {
                subcommand_groups.push(group!("week", subcommands));
            }
        }

        if !subcommand_groups.is_empty() {
            commands.push(Command {
                name: "top".to_string(),
                description: "-".to_string(),
                options: subcommand_groups,
                ..defaults_command()
            })
        }
    }
}

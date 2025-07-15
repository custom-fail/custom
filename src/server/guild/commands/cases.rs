use crate::models::config::GuildConfig;
use crate::server::guild::commands::{
    choice, default_option, defaults_command, if_enabled, options, subcommand,
};
use twilight_model::application::command::{
    Command, CommandOption, CommandOptionType, CommandOptionValue, CommandType,
};
use twilight_model::guild::Permissions;

pub(super) fn add_case_command(config: &GuildConfig, commands: &mut Vec<Command>) {
    let mut case_options = vec![];

    if_enabled!(
        config,
        case_last,
        case_options,
        subcommand!(
            "last",
            "Last punishment this user has received",
            vec![options!(
                "member",
                "The user that is subject to punishment that will be displayed",
                CommandOptionType::User,
                true
            )]
        )
    );

    if_enabled!(
        config,
        case_list,
        case_options,
        subcommand!(
            "list",
            "Shows history of user's punishment",
            vec![
                options!(
                    "member",
                    "The user that is subject to punishment that will be displayed",
                    CommandOptionType::User,
                    true
                ),
                CommandOption {
                    name: "page".to_string(),
                    description: "The page you want to retrieve".to_string(),
                    min_value: Some(CommandOptionValue::Integer(1)),
                    kind: CommandOptionType::Integer,
                    ..default_option()
                },
                CommandOption {
                    name: "type".to_string(),
                    description: "Moderation action you want to see".to_string(),
                    kind: CommandOptionType::String,
                    choices: Some(vec![
                        choice!("mutes"),
                        choice!("warns"),
                        choice!("kicks"),
                        choice!("bans"),
                    ]),
                    ..default_option()
                }
            ]
        )
    );

    if_enabled!(
        config,
        case_details,
        case_options,
        subcommand!(
            "details",
            "Shows detailed information about a case",
            vec![options!(
                "number",
                "The case number you want to see",
                CommandOptionType::Integer,
                true
            ),]
        )
    );

    if_enabled!(
        config,
        case_remove,
        case_options,
        subcommand!(
            "remove",
            "Removes a case from user's history",
            vec![options!(
                "number",
                "Number of a case you want to delete",
                CommandOptionType::Integer,
                true
            ),]
        )
    );

    if_enabled!(
        config,
        case_edit,
        case_options,
        subcommand!(
            "edit",
            "Edit case reason",
            vec![
                options!(
                    "number",
                    "Number of a case you want to edit",
                    CommandOptionType::Integer,
                    true
                ),
                CommandOption {
                    name: "reason".to_string(),
                    description: "New reason".to_string(),
                    kind: CommandOptionType::String,
                    required: Some(true),
                    max_length: Some(512),
                    ..default_option()
                }
            ]
        )
    );

    if !case_options.is_empty() {
        commands.push(Command {
            name: "case".to_string(),
            description: "-".to_string(),
            options: case_options,
            kind: CommandType::ChatInput,
            default_member_permissions: Some(Permissions::MODERATE_MEMBERS),
            ..defaults_command()
        })
    }
}

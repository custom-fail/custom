use std::collections::HashMap;
use twilight_model::application::interaction::application_command::{CommandData, CommandDataOption, CommandOptionValue};
use twilight_model::application::interaction::application_command::CommandOptionValue::{SubCommand, SubCommandGroup};

#[derive(Debug)]
pub struct CommandContext {
    options: HashMap<String, CommandOptionValue>,
    command_text: String,
}

impl CommandContext {
    pub fn from_command_data(command: CommandData) -> Self {
        let mut command_options = HashMap::new();
        for (name, value) in parse_options_to_value(command.options) {
            command_options.insert(name, value);
        }
        Self {
            options: command_options,
            command_text: "".to_string()
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
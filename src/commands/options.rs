use std::collections::HashMap;
use twilight_model::application::interaction::application_command::{CommandData, CommandDataOption, CommandOptionValue};
use twilight_model::application::interaction::InteractionData;
use crate::{extract, ok_or_break, ok_or_skip};
use async_trait::async_trait;
use crate::application::{Application, ConvertableOptionsList};
use crate::commands::context::InteractionContext;
use crate::utils::errors::Error;
use self::convert::convert_value_to_option;

#[macro_export]
macro_rules! get_option {
    ($value: expr, CommandOptionValue::String) => {
        if let Some(CommandOptionValue::String(value)) = $value {
            if value.is_empty() { None } else { Some(value) }
        } else { None }
    };
    ($value: expr, $t: path) => {
        if let Some($t(value)) = $value { Some(value) } else { None }
    };
}

#[macro_export]
macro_rules! get_required_option {
    ($value: expr, CommandOptionValue::String) => {
        get_option!($value, CommandOptionValue::String).ok_or(
            format!("Missing option: context.options.{}", stringify!($value))
        )?
    };
    ($value: expr, $t: path) => {
        get_option!($value, $t).ok_or(
            format!("Missing option: context.options.{}", stringify!($value))
        )?
    };
}

#[async_trait]
pub trait LoadOptions {
    async fn load_options(self, application: &Application) -> Result<InteractionContext, Error>;
}

#[async_trait]
impl LoadOptions for InteractionContext {
    async fn load_options(mut self, application: &Application) -> Result<Self, Error> {
        extract!(&self.orginal, data);
        match data {
            InteractionData::ApplicationCommand(command) => {
                self.options = get_options_from_command_data(command.to_owned());
            }
            InteractionData::MessageComponent(data) => {
                let custom_id = parse_custom_id(data.custom_id.to_owned());
                let name = custom_id.get(1).ok_or("Invalid custom_id")?;

                let component = application.find_component(&name.to_string())
                    .await.ok_or("Unknown component")?;
                self.command_text = component.command;

                for i in 0..data.values.len() {
                    let value = ok_or_break!(data.values.get(i), Some);
                    let (name, kind) = ok_or_break!(
                        component.values.get(i), Some
                    );
                    self.options.insert(
                        name.to_owned(),
                        convert_value_to_option(value.as_str(), &kind)?
                    );
                }

                set_options_from_custom_id(&mut self.options, custom_id, component.options)?;
            }
            InteractionData::ModalSubmit(data) => {
                let custom_id = parse_custom_id(data.custom_id.to_owned());
                let name = custom_id.get(1).ok_or("Invalid custom_id")?;

                let component = application.find_modal(&name.to_string())
                    .await.ok_or("Unknown modal")?;
                self.command_text = component.command;

                for action_row in &data.components {
                    for text_input in &action_row.components {
                        let kind = ok_or_skip!(
                            &component.inputs.get(text_input.custom_id.as_str()), Some
                        );
                        let value = convert_value_to_option(
                            ok_or_skip!(&text_input.value, Some).to_owned().as_str(), kind
                        )?;
                        self.options.insert(text_input.custom_id.clone(), value);
                    }
                }

                set_options_from_custom_id(&mut self.options, custom_id, component.options)?;
            }
            _ => return Err(Error::from("Invalid interaction data"))
        };

        Ok(self)
    }
}

fn set_options_from_custom_id(
    options: &mut HashMap<String, CommandOptionValue>,
    custom_id: Vec<String>,
    component: ConvertableOptionsList
) -> Result<(), Error> {
    for i in 2..custom_id.len() {
        let value = ok_or_break!(custom_id.get(i), Some);
        let (name, kind) = ok_or_break!(component.get(i - 2), Some);
        options.insert(name, convert_value_to_option(value.as_str(), &kind)?);
    }
    Ok(())
}

pub mod convert {
    use std::str::FromStr;
    use twilight_model::application::interaction::application_command::CommandOptionValue;
    use twilight_model::id::Id;
    use crate::utils::errors::Error;

    #[allow(dead_code)]

    #[derive(Copy, Clone)]
    pub enum ConvertableCommandOptionType {
        String,
        Boolean,
        Integer,
        User
    }

    /// Converts `String` value to `CommandOptionValue`
    pub fn convert_value_to_option(
        value: &str,
        kind: &ConvertableCommandOptionType
    ) -> Result<CommandOptionValue, Error> {
        Ok(match kind {
            ConvertableCommandOptionType::String => CommandOptionValue::String(value.to_string()),
            ConvertableCommandOptionType::Boolean => CommandOptionValue::Boolean(value == "true"),
            ConvertableCommandOptionType::Integer => {
                CommandOptionValue::Integer(
                    value.parse::<i64>().map_err(|_| "Invalid option type".to_string())?
                )
            }
            ConvertableCommandOptionType::User => {
                CommandOptionValue::User(
                    Id::from_str(value).map_err(|_| "Invalid option type".to_string())?
                )
            }
        })
    }
}

fn parse_custom_id(custom_id: String) -> Vec<String> {
    custom_id.split(':').map(&str::to_string).collect::<Vec<String>>()
}

fn get_options_from_command_data(data: Box<CommandData>) -> HashMap<String, CommandOptionValue> {
    let mut options = HashMap::new();
    for (name, value) in get_options_vec(data.options) {
        options.insert(name, value);
    }
    options
}

fn get_options_vec(options: Vec<CommandDataOption>) -> Vec<(String, CommandOptionValue)> {
    let mut results = vec![];
    for option in options {
        match option.value {
            CommandOptionValue::SubCommandGroup(data)
            | CommandOptionValue::SubCommand (data) => {
                results.append(&mut get_options_vec(data))
            },
            _ => results.push((option.name, option.value))
        };
    }
    results
}
use std::collections::HashMap;
use twilight_model::application::interaction::application_command::{CommandDataOption, CommandInteractionDataResolved};
use twilight_model::application::interaction::application_command::CommandOptionValue;
use twilight_model::application::interaction::{Interaction, InteractionData};
use twilight_model::id::Id;
use twilight_model::id::marker::GenericMarker;
use crate::utils::errors::Error;

#[macro_export]
macro_rules! extract {
    (&$t: expr, $($value: ident),*) => {
        $(
            let $value = $t.$value.as_ref().ok_or(
                format!("Missing value {}.{}", stringify!($t), stringify!($value))
            )?;
        )*
    };
    ($t: expr, $($value: ident),*) => {
        $(
            let $value = $t.$value.ok_or(
                format!("Missing value {}.{}", stringify!($t), stringify!($value))
            )?;
        )*
    };
}

pub struct InteractionContext {
    pub command_vec: Vec<String>,
    pub command_text: String,
    pub options: HashMap<String, CommandOptionValue>,
    pub interaction: Interaction
}

impl TryFrom<Interaction> for InteractionContext {
    type Error = Error;

    fn try_from(interaction: Interaction) -> Result<Self, Error> {
        extract!(&interaction, data);

        let command_vec = match data {
            InteractionData::ApplicationCommand(data) => {
                get_command_vec_from_application_command(
                    data.name.to_owned(), data.options.to_owned()
                )
            }
            InteractionData::MessageComponent(data) => {
                vec![get_command_from_id(&data.custom_id).ok_or("Invalid custom_id")?]
            }
            InteractionData::ModalSubmit(data) => {
                vec![get_command_from_id(&data.custom_id).ok_or("Invalid custom_id")?]
            }
            _ => return Err(Error::from("Invalid interaction type"))
        }.iter().map(|s| s.to_lowercase()).collect::<Vec<String>>();

        let command_text = command_vec.join(" ");

        Ok(Self {
            options: Default::default(),
            command_vec,
            command_text,
            interaction
        })
    }
}

fn get_command_from_id(custom_id: &str) -> Option<String> {
    let id_parts = custom_id.split(':').collect::<Vec<&str>>();
    id_parts.get(2).map(|d| d.to_string())
}

fn get_command_vec_from_application_command(name: String, options: Vec<CommandDataOption>) -> Vec<String> {
    options.get(0).map(|data| {
        let mut command_vec = vec![name.to_owned()];
        push_subcommands_names(&mut command_vec, data.to_owned());
        command_vec
    }).unwrap_or_else(|| vec![name.to_owned()])
}

fn push_subcommands_names(before: &mut Vec<String>, data: CommandDataOption) {
    let subcommand_value = match data.value {
        CommandOptionValue::SubCommandGroup(value) => Some(value),
        CommandOptionValue::SubCommand(value) => Some(value),
        _ => None
    };

    if let Some(options) = subcommand_value {
        before.push(data.name);
        let option = options.get(0).cloned();
        if let Some(option) = option {
            push_subcommands_names(before, option);
        }
    }
}

pub trait InteractionHelpers {
    fn resolved(&self) -> Option<&CommandInteractionDataResolved>;
    fn target_id(&self) -> Option<Id<GenericMarker>>;
}

impl InteractionHelpers for Interaction {
    fn resolved(&self) -> Option<&CommandInteractionDataResolved> {
        if let Some(InteractionData::ApplicationCommand(data)) = &self.data {
            data.resolved.as_ref()
        } else { None }
    }

    fn target_id(&self) -> Option<Id<GenericMarker>> {
        if let Some(InteractionData::ApplicationCommand(data)) = &self.data {
            data.target_id.as_ref().copied()
        } else { None }
    }
}

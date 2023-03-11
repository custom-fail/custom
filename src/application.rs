use std::collections::HashMap;
use std::sync::Arc;
use crate::command;
use crate::commands::Command;
use futures_util::FutureExt;
use crate::models::config::GuildConfig;
use twilight_http::Client;
use crate::context::Context;
use crate::commands::context::InteractionContext;
use crate::commands::options::convert::ConvertableCommandOptionType;

pub type ConvertableOptionsHashMap = HashMap<String, ConvertableCommandOptionType>;
pub type ConvertableOptionsList = Vec<(String, ConvertableCommandOptionType)>;

#[derive(Clone)]
pub struct Component {
    pub options: ConvertableOptionsList,
    pub values: ConvertableOptionsList,
    pub command: String,
    pub id: String
}

#[derive(Clone)]
pub struct Modal {
    pub options: ConvertableOptionsList,
    pub inputs: ConvertableOptionsHashMap,
    pub command: String,
    pub id: String
}

#[derive(Clone)]
pub struct Application {
    commands: HashMap<String, Command>,
    components: HashMap<String, Component>,
    modals: HashMap<String, Modal>,
    slower_commands: Vec<String>
}

fn get_modal_input(duration: bool, dashboard: bool) -> ConvertableOptionsHashMap {
    let mut all = vec![];
    if dashboard { all.push(("member".to_string(), ConvertableCommandOptionType::User)) };
    if duration { all.push(("duration".to_string(), ConvertableCommandOptionType::String)) };
    all.push(("reason".to_string(), ConvertableCommandOptionType::String));
    HashMap::from_iter(all.into_iter())
}

macro_rules! moderation_modal {
    ([$($name: expr),*], [$($duration_command_name: expr),*]) => {
        HashMap::from([$(
            ($name.to_string(), Modal {
                options: vec![("member".to_string(), ConvertableCommandOptionType::User)],
                inputs: get_modal_input(false, false),
                command: $name.to_string(),
                id: $name.to_string(),
            }),
            (format!("{}-d", $name), Modal {
                options: vec![],
                inputs: get_modal_input(false, true),
                command: $name.to_string(),
                id: format!("{}-d", $name)
            }),
        )*
        $(
            ($duration_command_name.to_string(), Modal {
                options: vec![("member".to_string(), ConvertableCommandOptionType::User)],
                inputs: get_modal_input(true, false),
                command: $duration_command_name.to_string(),
                id: $duration_command_name.to_string(),
            }),
            (format!("{}-d", $duration_command_name), Modal {
                options: vec![],
                inputs: get_modal_input(true, true),
                command: $duration_command_name.to_string(),
                id: format!("{}-d", $duration_command_name)
            }),
        )*])
    }
}

macro_rules! set_command {
    ($name: expr, $module: expr, $run: expr) => {
        ($name.to_string(), command!($name.to_string(), $module.to_string(), $run))
    };
}

impl Application {
    pub fn new() -> Self {
        let commands = HashMap::from([
            set_command!("case details", "moderation", crate::commands::case::details::run),
            set_command!("case remove", "moderation", crate::commands::case::remove::run),
            set_command!("case edit", "moderation", crate::commands::case::edit::run),
            set_command!("case last", "moderation", crate::commands::case::last::run),
            set_command!("case list", "moderation", crate::commands::case::list::run),

            set_command!("timeout", "moderation", crate::commands::moderation::execute::run),
            set_command!("kick", "moderation", crate::commands::moderation::execute::run),
            set_command!("mute", "moderation", crate::commands::moderation::execute::run),
            set_command!("warn", "moderation", crate::commands::moderation::execute::run),
            set_command!("ban", "moderation", crate::commands::moderation::execute::run),

            set_command!("mod-dash", "moderation", crate::commands::moderation::dash::run),
            set_command!("clear", "moderation", crate::commands::moderation::clear::run),

            set_command!("top week all", "top", crate::commands::top::all::run),
            set_command!("top day all", "top", crate::commands::top::all::run),
            set_command!("top week me", "top", crate::commands::top::me::run),
            set_command!("top day me", "top", crate::commands::top::me::run),

            set_command!("setup", "settings", crate::commands::settings::setup::run)
        ]);

        let components = HashMap::from([
            ("cl".to_string(), Component {
                options: vec![("member".to_string(), ConvertableCommandOptionType::User)],
                values: vec![("page".to_string(), ConvertableCommandOptionType::Integer)],
                command: "case list".to_string(),
                id: "cl".to_string()
            }),
            ("mod-panel".to_string(), Component {
                options: vec![("action".to_string(), ConvertableCommandOptionType::String)],
                values: vec![],
                command: "mod-dash".to_string(),
                id: "mod-panel".to_string()
            })
        ]);

        let modals = moderation_modal!(["warn", "kick", "ban"], ["timeout", "mute"]);

        let slower_commands = vec!["kick", "mute", "warn", "ban", "clear", "case list"]
                .iter().map(|c| c.to_string()).collect();

        Self {
            commands,
            components,
            modals,
            slower_commands
        }
    }

    pub async fn is_slower(&self, command: &String) -> bool {
        self.slower_commands.contains(command)
    }

    pub async fn find_command(&self, name: &String) -> Option<Command> {
        self.commands.get(name).cloned()
    }

    pub async fn find_component(&self, id: &String) -> Option<Component> {
        self.components.get(id).cloned()
    }

    pub async fn find_modal(&self, id: &String) -> Option<Modal> {
        self.modals.get(id).cloned()
    }
}
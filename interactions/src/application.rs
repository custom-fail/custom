use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::commands::Command;

#[derive(Clone)]
pub struct Component {
    pub options: Vec<(String, String)>,
    pub values: Vec<(String, String)>,
    pub command: String,
    pub id: String
}

#[derive(Clone)]
pub struct Application {
    commands: Arc<Mutex<HashMap<String, Command>>>,
    components: Arc<Mutex<HashMap<String, Component>>>
}

impl Application {
    pub fn new() -> Self {
        Self {
            commands: Arc::new(Mutex::new(HashMap::new())),
            components: Arc::new(Mutex::new(HashMap::new()))
        }
    }

    pub async fn add_commands(&self, commands: Vec<Command>) {
        let mut cmds = self.commands.lock().await;
        for command in commands.iter() {
            cmds.insert(command.name.clone(), command.to_owned());
        }
    }

    pub async fn find_command(&self, name: String) -> Option<Command> {
        let commands = self.commands.lock().await;
        commands.get(&name).cloned()
    }

    pub async fn add_components(&self, components: Vec<Component>) {
        let mut cmds = self.components.lock().await;
        for component in components.iter() {
            cmds.insert(component.id.clone(), component.to_owned());
        }
    }

    pub async fn find_component(&self, id: String) -> Option<Component> {
        let components = self.components.lock().await;
        components.get(&id).cloned()
    }

}
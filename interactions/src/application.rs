use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::commands::Command;

pub struct Application {
    commands: Arc<Mutex<HashMap<String, Command>>>,
}

impl Application {
    pub fn new() -> Self {
        Self {
            commands: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn add_command(&mut self, commands: Vec<Command>) {
        let mut cmds = self.commands.lock().await;
        for command in commands.iter() {
            cmds.insert(command.name.clone(), command.to_owned());
        }
    }

    pub async fn find_command(&self, name: String) -> Option<Command> {
        let commands = self.commands.lock().await;
        commands.get(&name).cloned()
    }

}

impl Clone for Application {
    fn clone(&self) -> Self {
        Self {
            commands: self.commands.clone()
        }
    }
}
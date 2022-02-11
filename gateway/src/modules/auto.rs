use std::sync::Arc;
use twilight_http::Client;
use twilight_http::request::AuditLogReason;
use twilight_model::gateway::payload::incoming::MessageCreate;
use database::models::config::{Action, AutoModerator};
use database::mongodb::MongoDBConnection;

async fn execute_action(discord_http: Arc<Client>, action: Action, message: Box<MessageCreate>, reason: &str) -> Result<(), ()> {
    if action.delete_message {
        discord_http.delete_message(message.channel_id, message.id).reason(reason).map_err(|_| ())?.exec().await;
    };
    Ok(())
}

pub async fn run(message: Box<MessageCreate>, mongodb: MongoDBConnection, discord_http: Arc<Client>) -> Result<(), ()> {

    let guild_id = message.guild_id.ok_or(())?;
    let guild_config = mongodb.get_config(guild_id).await.map_err(|_| ())?;

    for automod_config in guild_config.moderation.automod {
        match automod_config {
            AutoModerator::MessageLength(config) => {
                let lines = message.content.len() / config.line_len as usize + message.content.find("\n").unwrap_or(1);
                if lines < config.max_lines as usize { continue }
                execute_action(
                    discord_http.clone(),
                    guild_config.moderation.automod_actions.get(config.first_action.as_str()).ok_or(())?.clone(),
                    message.clone(),
                    "Sending too long messages"
                ).await?;
            }
            AutoModerator::AntiCapsLock(_) => {}
        }
    }

    Ok(())

}
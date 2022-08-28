use std::sync::Arc;
use std::time::Duration;
use mongodb::bson::DateTime;
use tokio::time::Instant;
use twilight_http::Client;
use twilight_model::id::Id;
use twilight_model::id::marker::RoleMarker;
use crate::{DiscordClients, MongoDBConnection, ok_or_skip};
use crate::models::config::GuildConfig;
use crate::models::task::{Task, TaskAction};

pub fn run(
    mongodb: MongoDBConnection,
    discord_clients: DiscordClients,
    discord_http: Arc<Client>
) {
    tokio::spawn(interval(mongodb, discord_clients, discord_http));
}

pub async fn interval(
    mongodb: MongoDBConnection,
    discord_clients: DiscordClients,
    discord_http: Arc<Client>
) {
    loop {
        let tasks = mongodb.get_and_delete_future_tasks(60 * 1000).await;

        if let Ok(tasks) = tasks {
            if !tasks.is_empty() {
                println!("Loaded {} tasks", tasks.len())
            };

            for task in tasks {
                let guild_config = ok_or_skip!(mongodb.get_config(task.guild_id).await, Ok);
                let guild_discord_http = guild_config.application_id
                    .and_then(|id| {
                        discord_clients.get(&id)
                            .map(|http| http.to_owned())
                    }).unwrap_or_else(|| discord_http.to_owned());

                tokio::spawn(execute_task(task, guild_config, guild_discord_http));
            }
        }

        tokio::time::sleep(Duration::from_secs(60)).await;
    };
}

pub async fn execute_task(task: Task, config: GuildConfig, discord_http: Arc<Client>) {
    let execute_in = u64::try_from(
        task.execute_at.timestamp_millis() - DateTime::now().timestamp_millis()
    ).unwrap_or(0); // If number is negative set it to 0 (execute it now)
    tokio::time::sleep_until(Instant::now() + Duration::from_millis(execute_in)).await;

    run_action(task, config, discord_http).await.ok();
}

pub async fn run_action(task: Task, config: GuildConfig, discord_http: Arc<Client>) -> Result<(), ()> {
    match task.action {
        TaskAction::RemoveMuteRole(member_id) => {
            let member = discord_http.guild_member(config.guild_id, member_id)
                .exec().await.map_err(|_| ())?.model().await.map_err(|_| ())?;

            let mute_role = config.moderation.mute_role.ok_or(())?;
            let roles_without_mute_role = member.roles.iter()
                .filter(|role| role != &&mute_role).cloned().collect::<Vec<Id<RoleMarker>>>();

            discord_http.update_guild_member(config.guild_id, member_id)
                .roles(&roles_without_mute_role).exec().await.map_err(|_| ())?;
        }
        TaskAction::RemoveBan(member_id) => {
            discord_http.delete_ban(config.guild_id, member_id).exec().await.map_err(|_| ())?;
        }
    };
    Ok(())
}
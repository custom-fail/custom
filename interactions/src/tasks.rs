use std::sync::Arc;
use std::time::Duration;
use database::clients::DiscordClients;
use database::models::config::GuildConfig;
use database::models::task::{Task, TaskAction};
use database::mongodb::MongoDBConnection;
use mongodb::bson::DateTime;
use tokio::time::Instant;
use twilight_http::Client;
use utils::ok_or_skip;

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
                    .map(|id| {
                        discord_clients.get(&id)
                            .map(|client| client.http.to_owned())
                    }).flatten().unwrap_or_else(|| discord_http.to_owned());

                tokio::spawn(execute_task(task, guild_config, guild_discord_http));
            }
        }

        tokio::time::sleep(Duration::from_secs(60)).await;
    };
}

pub async fn execute_task(task: Task, _: GuildConfig, _: Arc<Client>) {
    let execute_in = u64::try_from(
        task.execute_at.timestamp_millis() - DateTime::now().timestamp_millis()
    ).unwrap_or(0); // If number is negative set it to 0 (execute it now)
    tokio::time::sleep_until(Instant::now() + Duration::from_millis(execute_in)).await;

    match task.action {
        TaskAction::RemoveMuteRole(_) => {
            todo!("New muting system")
        }
    };

}
use std::sync::Arc;
use twilight_http::Client;
use database::models::config::GuildConfig;
use database::mongodb::MongoDBConnection;
use database::redis::RedisConnection;
use utils::embeds::EmbedBuilder;
use crate::commands::context::InteractionContext;
use crate::commands::ResponseData;

const PLACES_EMOTES: [&str; 3] = [":first_place:", ":second_place:", ":third_place:"];

pub async fn run(interaction: InteractionContext, _: MongoDBConnection, redis: RedisConnection, _: Arc<Client>, _: GuildConfig) -> ResponseData {

    let guild_id = interaction.guild_id.ok_or("Cannot find guild_id")?;

    let week_or_day = interaction.command_vec.get(1).cloned()
        .ok_or("Invalid command".to_string())?;
    if !["week", "day"].contains(&week_or_day.clone().as_str()) {
        return Err("Invalid command".to_string())
    }

    let leaderboard = redis.get_all(format!("top_{week_or_day}.{guild_id}"), 3).map_err(|err| format!("{err}"))?;

    let leaderboard_string = leaderboard
        .clone()
        .iter()
        .enumerate()
        .map(|(index, (user_id, messages))| -> String {
            format!("{} > <@{user_id}> ({messages})", PLACES_EMOTES[index])
        })
        .collect::<Vec<String>>()
        .join("\n");

    Ok((
        EmbedBuilder::new()
            .title(format!("Top {week_or_day} users"))
            .description(leaderboard_string)
            .to_interaction_response_data(),
        None
    ))

}
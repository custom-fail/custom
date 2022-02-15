use std::sync::Arc;
use twilight_http::Client;
use twilight_model::application::callback::CallbackData;
use twilight_model::application::interaction::ApplicationCommand;
use database::mongodb::MongoDBConnection;
use database::redis::RedisConnection;
use crate::utilities::embed::text_to_response_embed;

const PLACES_EMOTES: [&str; 3] = [":first_place:", ":second_place:", ":third_place:"];

pub async fn run(interaction: Box<ApplicationCommand>, _: MongoDBConnection, redis: RedisConnection, _: Arc<Client>) -> Result<CallbackData, String> {

    let guild_id = interaction.guild_id.ok_or("Cannot find guild_id")?;

    let week_or_day = interaction.data.options.first().cloned().ok_or("Invalid command".to_string())?.name;
    if !["week", "day"].contains(&week_or_day.clone().as_str()) {
        return Err("Invalid command".to_string())
    }

    let leaderboard = redis.get_all(format!("top_{week_or_day}.{guild_id}"), 3).map_err(|err| format!("{err}"))?;

    let leaderboard_string = leaderboard
        .clone()
        .iter()
        .enumerate()
        .map(|all| -> String {
            let top = all.1;
            let index = all.0;
            format!("{} > <@{}> ({})", PLACES_EMOTES[index], top.0, top.1)
        })
        .collect::<Vec<String>>()
        .join("\n");

    Ok(text_to_response_embed(format!("Top {week_or_day} users"), leaderboard_string))

}
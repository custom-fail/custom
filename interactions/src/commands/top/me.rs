use std::sync::Arc;
use twilight_http::Client;
use twilight_model::application::callback::CallbackData;
use twilight_model::application::interaction::ApplicationCommand;
use database::mongodb::MongoDBConnection;
use database::redis::RedisConnection;
use crate::utilities::embed::text_to_response_embed;

pub async fn run(interaction: Box<ApplicationCommand>, _: MongoDBConnection, redis: RedisConnection, _: Arc<Client>) -> Result<CallbackData, String> {

    let guild_id = interaction.guild_id.ok_or("Cannot find guild_id")?;
    let user = interaction.member.ok_or("Cannot find member")?.user.ok_or("Unknown user")?;

    let week_or_day = interaction.data.options.first().cloned().ok_or("Invalid command".to_string())?.name;
    if !["week", "day"].contains(&week_or_day.clone().as_str()) {
        return Err("Invalid command".to_string())
    }

    let (user_score, user_position) = redis.get_by_user(
        format!("top_{}.{}", week_or_day, guild_id), user.id
    ).map_err(|err| format!("{}", err))?;

    let mut result = format!("You are **{}** with **{}** messages", user_position + 1, user_score);

    let leaderboard = redis.get_all(
        format!("top_{}.{}", week_or_day, guild_id), 3
    ).map_err(|err| format!("{}", err))?;

    let leaderboard_string = leaderboard.iter().enumerate()
        .map(|(index, top)|
            if user_position <= (index as u32) {
                format!("You are **{}** messages behind **{}** place", user_score - top.1, index + 1) }
            else { format!("You need **{}** messages to be **{}**", top.1 - user_score, index + 1) }
        )
        .collect::<Vec<String>>().join("\n");

    result = format!("{}\n\n{}\n\n", result, leaderboard_string);

    if user_position > 0 {
        let user_after = redis.get_by_position(
            format!("top_{}.{}", week_or_day, guild_id), (user_position - 1) as usize
        ).map_err(|err| format!("{}", err))?.ok_or("There is no `user_after`")?;
        result = format!("{}**{}** messages to beat next user\n", result, user_after);
    }

    let user_before = redis.get_by_position(
        format!("top_{}.{}", week_or_day, guild_id), (user_position + 1) as usize
    ).map_err(|err| format!("{}", err))?.ok_or("There is no `user_before`")?;
    result = format!("{}**{}** messages for user before (to you)", result, user_before);

    Ok(text_to_response_embed(
        format!("Top of the {} for {}#{}", week_or_day, user.name, user.discriminator),
        result
    ))

}
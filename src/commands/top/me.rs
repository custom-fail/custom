use std::sync::Arc;
use twilight_http::Client;
use crate::utils::embeds::EmbedBuilder;
use crate::utils::errors::Error;
use crate::commands::context::InteractionContext;
use crate::commands::ResponseData;
use crate::{extract, MongoDBConnection, RedisConnection};
use crate::models::config::GuildConfig;

pub async fn run(
    context: InteractionContext,
    _: MongoDBConnection,
    redis: RedisConnection,
    _: Arc<Client>,
    _: GuildConfig
) -> ResponseData {
    extract!(context.interaction, guild_id, member);
    extract!(member, user);

    let week_or_day = context.command_vec.get(1).cloned()
        .ok_or("Invalid command")?;
    if !["week", "day"].contains(&week_or_day.as_str()) {
        return Err(Error::from("Invalid command"))
    }

    let (user_score, user_position) = redis.get_by_user(
        format!("top_{week_or_day}.{guild_id}"), user.id
    ).map_err(Error::from)?;

    let mut result = format!("You are **{}** with **{user_score}** messages", user_position + 1);

    let leaderboard = redis.get_all(
        format!("top_{week_or_day}.{guild_id}"), 3
    ).map_err(Error::from)?;

    let leaderboard_string = leaderboard.iter().enumerate()
        .map(|(index, top)|
            if user_position <= (index as u32) {
                format!("You are **{}** messages behind **{}** place", user_score - top.1, index + 1) }
            else { format!("You need **{}** messages to be **{}**", top.1 - user_score, index + 1) }
        )
        .collect::<Vec<String>>().join("\n");

    result = format!("{result}\n\n{leaderboard_string}\n\n");

    if user_position > 0 {
        let user_after = redis.get_by_position(
            format!("top_{week_or_day}.{guild_id}"), (user_position - 1) as usize
        ).map_err(Error::from)?.ok_or("There is no user_after")?;
        result = format!("{result}**{user_after}** messages to beat next user\n");
    }

    let user_before = redis.get_by_position(
        format!("top_{week_or_day}.{guild_id}"), (user_position + 1) as usize
    ).map_err(Error::from)?.ok_or("There is no `user_before`")?;
    result = format!("{result}**{user_before}** messages for user before (to you)");

    Ok((
        EmbedBuilder::new()
            .title(
                format!("Top of the {week_or_day} for {}#{}", user.name, user.discriminator)
            )
            .description(result)
            .to_interaction_response_data(false),
        None
    ))
}
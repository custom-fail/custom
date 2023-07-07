use crate::commands::context::InteractionContext;
use crate::commands::ResponseData;
use crate::context::Context;
use crate::models::config::GuildConfig;
use crate::utils::errors::Error;
use crate::{extract, render_context};
use std::sync::Arc;
use twilight_http::Client;

pub async fn run(
    interaction: InteractionContext,
    context: Arc<Context>,
    _: Arc<Client>,
    config: GuildConfig,
) -> ResponseData {
    extract!(interaction.orginal, guild_id);
    let user_id = interaction
        .orginal
        .author_id()
        .ok_or("Cannot find user information")?;

    let week_or_day = interaction
        .command_vec
        .get(1)
        .cloned()
        .ok_or("Invalid command")?;
    if !["week", "day"].contains(&week_or_day.as_str()) {
        return Err(Error::from("Invalid command"));
    }

    let (user_score, user_position) = context
        .redis
        .get_by_user(format!("top_{week_or_day}.{guild_id}"), user_id)
        .map_err(Error::from)?;

    let leaderboard = context
        .redis
        .get_all(format!("top_{week_or_day}.{guild_id}"), 3)
        .map_err(Error::from)?;

    let user_after = if user_position > 0 {
        context
            .redis
            .get_by_position(
                format!("top_{week_or_day}.{guild_id}"),
                (user_position - 1) as usize,
            )
            .map_err(Error::from)?
    } else {
        None
    };

    let user_before = context
        .redis
        .get_by_position(
            format!("top_{week_or_day}.{guild_id}"),
            (user_position + 1) as usize,
        )
        .map_err(Error::from)?;

    Ok((
        config
            .assets
            .render_message(
                &context.assets,
                "commands.top.all",
                &mut render_context!(
                    ["interaction", &interaction.orginal],
                    ["leaderboard", &leaderboard],
                    ["weekOrDay", &week_or_day],
                    ["userPosition", &user_position],
                    ["userScore", &user_score],
                    ["userBefore", &user_before],
                    ["userAfter", &user_after]
                ),
                &context.redis,
            )
            .await?,
        None,
    ))
}

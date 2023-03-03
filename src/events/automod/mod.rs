pub mod actions;
mod checks;
mod filters;

use std::sync::Arc;
use twilight_http::Client;
use twilight_model::channel::Message;
use crate::context::Context;
use crate::models::config::moderation::Ignore;
use crate::events::automod::actions::run_action;
use self::filters::filters_match;
use self::checks::checks_match;

fn is_ignored(message: &Message, ignores: &Vec<Ignore>) -> bool {

    let member = match message.member.to_owned() {
        Some(member) => member,
        None => return false
    };

    for role in member.roles {
        if ignores.contains(&Ignore::Role(role)) {
            return true
        }
    }

    if ignores.contains(&Ignore::Channel(message.channel_id))
        || ignores.contains(&Ignore::User(message.author.id)) {
        return true
    }

    false

}

pub async fn run(
    message: Message,
    discord_http: Arc<Client>,
    context: Arc<Context>
) -> Result<(), ()> {
    let guild_id = message.guild_id.ok_or(())?;
    let guild_config = context.mongodb.get_config(guild_id).await.map_err(|_| ())?;

    if message.content.is_empty() || message.author.bot {
        return Ok(())
    }

    if is_ignored(&message, &guild_config.moderation.automod_ignore) { return Ok(()) }

    for automod in guild_config.moderation.automod.to_owned() {

        if is_ignored(&message, &automod.ignore) { continue }

        for filter in automod.filters {
            let is_filtered = filters_match(filter, message.to_owned());
            if is_filtered { return Ok(()) }
        }

        for check in automod.checks {
            let is_allowed = checks_match(check, message.content.to_owned(), context.scam_domains.to_owned()).await?;
            if !is_allowed { return Ok(()) }
        }

        for action in automod.actions {
            run_action(
                action,
                message.to_owned(),
                discord_http.to_owned(),
                context.bucket.to_owned(),
                &guild_config,
                automod.reason.to_owned(),
            ).await.ok();
        }

    }

    Ok(())
}
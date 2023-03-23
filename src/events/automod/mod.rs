pub mod actions;
mod checks;
mod filters;

use std::sync::Arc;
use twilight_http::Client;
use twilight_model::channel::Message;
use crate::context::Context;
use crate::events::automod::actions::run_action;
use crate::models::config::automod::TrigerEvent;
use crate::models::config::automod::ignore::{Ignore, IgnoreMode};

fn is_ignored(message: &Message, ignore_rule: &Option<Ignore>) -> bool {
    let ignore_rule = match ignore_rule {
        Some(rule) => rule,
        None => return false
    };

    let member = match &message.member {
        Some(member) => member,
        None => return false
    };

    for role in &member.roles {
        if ignore_rule.roles.contains(role) { return true }
    }

    if ignore_rule.users.contains(&message.author.id) { return true }

    let is_whitelist = ignore_rule.channels_ignore_mode == IgnoreMode::WhileList;
    let contains_channel = ignore_rule.channels.contains(&message.channel_id);

    (is_whitelist && !contains_channel) || (!is_whitelist && contains_channel)
}

pub async fn run(
    message: Message,
    discord_http: Arc<Client>,
    context: Arc<Context>,
    triger: TrigerEvent
) -> Result<(), ()> {
    let guild_id = message.guild_id.ok_or(())?;
    let guild_config = Arc::new(context.mongodb.get_config(guild_id).await.map_err(|_| ())?);

    if message.content.is_empty() || message.author.bot {
        return Ok(())
    }

    if is_ignored(&message, &guild_config.moderation.automod.ignore) { return Ok(()) }

    let message = Arc::new(message);

    for automod_rule in &guild_config.moderation.automod.rules {
        if triger == TrigerEvent::MessageUpdate && !automod_rule.check_on_edit { continue }
        if is_ignored(&message, &automod_rule.ignore) { continue }

        for filter_meta in &automod_rule.filters {
            if filter_meta.filter.is_matching(&message) != filter_meta.negate { return Ok(()) }
        }

        for check in &automod_rule.checks {
            let is_allowed = check.is_matching(&message.content, &context.scam_domains).await?;
            if !is_allowed { return Ok(()) }
        }

        for action in &automod_rule.actions {
            let run = run_action(
                action.action.to_owned(),
                message.to_owned(),
                discord_http.to_owned(),
                context.bucket.to_owned(),
                guild_config.to_owned(),
                automod_rule.reason.to_owned(),
            );

            if action.sync {
                run.await.ok();
            } else { tokio::spawn(run); }
        }
    }

    Ok(())
}
pub mod actions;
mod checks;
mod filters;

use std::sync::Arc;
use twilight_http::Client;
use twilight_model::channel::Message;
use database::mongodb::MongoDBConnection;
use crate::modules::automod::actions::run_action;
use crate::{Bucket, ScamLinks};
use self::filters::filters_match;
use self::checks::checks_match;

pub async fn run(message: Message, mongodb: MongoDBConnection, discord_http: Arc<Client>, scam_domains: ScamLinks, bucket: Bucket) -> Result<(), ()> {

    let guild_id = message.guild_id.ok_or(())?;
    let guild_config = mongodb.get_config(guild_id).await.map_err(|_| ())?;

    if message.content.is_empty() || message.author.bot {
        return Ok(())
    }

    for automod in guild_config.moderation.automod.to_owned() {

        for filter in automod.filters {
            let is_filtered = filters_match(filter, message.to_owned());
            if is_filtered { return Ok(()) }
        }

        for check in automod.checks {
            let is_allowed = checks_match(check, message.content.to_owned(), scam_domains.to_owned()).await?;
            if !is_allowed { return Ok(()) }
        }

        for action in automod.actions {
            run_action(
                action,
                message.to_owned(),
                discord_http.to_owned(),
                bucket.to_owned(),
                &guild_config,
                automod.reason.to_owned(),
            ).await.ok();
        }

    }

    Ok(())

}
use std::sync::Arc;
use twilight_http::Client;
use twilight_http::request::AuditLogReason;
use twilight_model::channel::embed::{Embed, EmbedAuthor};
use twilight_model::gateway::payload::incoming::MessageCreate;
use twilight_model::id::Id;
use twilight_model::id::marker::UserMarker;
use twilight_model::util::ImageHash;
use database::models::config::automod::checks::Checks;
use database::models::config::automod::filters::Filters;
use database::models::config::moderation::Moderation;
use database::mongodb::MongoDBConnection;
use crate::{ok_or_skip, ScamLinks};

pub async fn run(message: Box<MessageCreate>, mongodb: MongoDBConnection, discord_http: Arc<Client>, scam_domains: ScamLinks) -> Result<(), ()> {

    let invites = regex::Regex::new(r"(?i)(discord.gg|discordapp.com/invite|discord.com/invite)(?:/#)?/([a-zA-Z0-9-]+)").map_err(|_| ())?;
    let domains = regex::Regex::new(r"(?:[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?\.)+[a-z0-9][a-z0-9-]{0,61}[a-z0-9]").map_err(|_| ())?;

    let guild_id = message.guild_id.ok_or(())?;
    let guild_config = mongodb.get_config(guild_id).await.map_err(|_| ())?;

    if message.content.len() == 0 || message.author.bot {
        return Ok(())
    }

    let message_content = message.content.to_lowercase();

    for automod in guild_config.moderation.automod {
        for filter in automod.filters {
            let is_filtred = match filter {
                Filters::Attachments(config) => {
                    let message_attachments_count = message.attachments.len();
                    (if let Some(min) = config.min {
                        (min as usize) > message_attachments_count
                    } else { false }) || (if let Some(max) = config.max {
                        (max as usize) < message_attachments_count
                    } else { false })
                },
                Filters::MessageLength(config) => {
                    let message_content_len = message.content.len();
                    (if let Some(min) = config.min {
                        (min as usize) > message_content_len
                    } else { false }) || (if let Some(max) = config.max {
                        (max as usize) < message_content_len
                    } else { false })
                },
                Filters::Stickers => { message.sticker_items.len() > 0 }
            };

            if is_filtred { return Ok(()) }
        }
    }

    Ok(())

}
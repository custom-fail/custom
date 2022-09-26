use std::sync::Arc;
use twilight_http::Client;
use twilight_model::channel::message::MessageFlags;
use twilight_model::http::interaction::InteractionResponseData;
use crate::commands::context::InteractionContext;
use crate::{extract, MongoDBConnection, RedisConnection};
use crate::commands::ResponseData;
use crate::models::config::GuildConfig;

pub async fn run(
    context: InteractionContext,
    _: MongoDBConnection,
    _: RedisConnection,
    _: Arc<Client>,
    _: GuildConfig
) -> ResponseData {
    extract!(context.interaction, guild_id);
    Ok((InteractionResponseData {
        allowed_mentions: None,
        attachments: None,
        choices: None,
        components: None,
        content: Some(
            format!("**The server setup is not completed yet or commands are not synced**\n\nTo complete a setup open the dashboard https://custom.fail/setup?guild={guild_id}\nTo sync commands open a server settings https://custom.fail/servers/{guild_id}")
        ),
        custom_id: None,
        embeds: None,
        flags: Some(MessageFlags::EPHEMERAL),
        title: None,
        tts: None
    }, None))
}
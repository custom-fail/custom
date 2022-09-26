use std::sync::Arc;
use mongodb::bson::doc;
use twilight_http::Client;
use twilight_model::application::interaction::application_command::CommandOptionValue;
use twilight_model::channel::message::MessageFlags;
use twilight_model::http::interaction::InteractionResponseData;
use crate::commands::context::InteractionContext;
use crate::commands::ResponseData;
use crate::{extract, get_required_option, get_option, MongoDBConnection, RedisConnection};
use crate::models::config::GuildConfig;
use crate::utils::errors::Error;

pub async fn run(
    context: InteractionContext,
    mongodb: MongoDBConnection,
    _: RedisConnection,
    discord_http: Arc<Client>,
    _: GuildConfig
) -> ResponseData {

    extract!(context.interaction, member, guild_id);
    extract!(member, user);

    let case_index = get_required_option!(
        context.options.get("number"), CommandOptionValue::Integer
    );

    let reason = get_required_option!(
        context.options.get("reason"), CommandOptionValue::String
    ).to_owned();

    if reason.len() > 512 {
        return Err(Error::from("Reason is too long"))
    }

    let mut case = mongodb.cases.find_one(
        doc! { "guild_id": guild_id.to_string(), "index": case_index, "removed": false }, None
    ).await.map_err(Error::from)?.ok_or("There is no case with selected id")?;

    if case.moderator_id != user.id {
        return Err(Error::from("You can't edit cases created by someone else"))
    }

    mongodb.cases.update_one(
        doc! { "guild_id": guild_id.to_string(), "index": case_index, "removed": false },
        doc! { "$set": {"reason": reason.to_owned() } }, None
    ).await.map_err(Error::from)?;

    case.reason = Some(reason);

    Ok((InteractionResponseData {
        allowed_mentions: None,
        attachments: None,
        choices: None,
        components: None,
        content: Some("**Case updated**".to_string()),
        custom_id: None,
        embeds: Some(vec![case.to_embed(discord_http).await?]),
        flags: Some(MessageFlags::EPHEMERAL),
        title: None,
        tts: None
    }, None))

}
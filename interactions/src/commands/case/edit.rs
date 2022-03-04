use std::sync::Arc;
use mongodb::bson::doc;
use twilight_http::Client;
use twilight_model::application::interaction::application_command::CommandOptionValue;
use twilight_model::channel::message::MessageFlags;
use database::mongodb::MongoDBConnection;
use database::redis::RedisConnection;
use twilight_model::http::interaction::InteractionResponseData;
use crate::check_type;
use crate::commands::context::InteractionContext;

pub async fn run(interaction: InteractionContext, mongodb: MongoDBConnection, _: RedisConnection, discord_http: Arc<Client>) -> Result<InteractionResponseData, String> {

    let guild_id = interaction.guild_id.ok_or("Cannot find guild_id".to_string())?;

    let case_id = check_type!(
        interaction.options.get("id").ok_or("There is no case id".to_string())?,
        CommandOptionValue::Integer
    ).ok_or("Case id type not match".to_string())?.clone();

    let mut case = mongodb.cases.find_one(
        doc! { "guild_id": guild_id.to_string(), "index": case_id, "removed": false }, None
    ).await.map_err(|err| format!("{err}"))?.ok_or("There is no case with selected id".to_string())?;

    let member_id = interaction.user.ok_or("Cannot get user data".to_string())?.id;

    if case.moderator_id != member_id {
        return Err("You can't edit cases created by someone else".to_string())
    }

    let reason = check_type!(
        interaction.options.get("reason").ok_or("There is no reason".to_string())?,
       CommandOptionValue::String
    ).ok_or("Reason type not match".to_string())?.clone();

    mongodb.cases.update_one(
        doc! { "guild_id": guild_id.to_string(), "index": case_id, "removed": false }, doc! { "$set": {"reason": reason.clone() } }, None
    ).await.map_err(|err| format!("{err}"))?;

    case.reason = Some(reason);

    Ok(InteractionResponseData {
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
    })

}
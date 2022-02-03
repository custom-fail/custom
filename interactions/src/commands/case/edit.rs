use std::sync::Arc;
use mongodb::bson::doc;
use twilight_http::Client;
use twilight_model::application::callback::CallbackData;
use twilight_model::application::interaction::application_command::CommandOptionValue;
use twilight_model::application::interaction::ApplicationCommand;
use twilight_model::channel::message::MessageFlags;
use database::mongodb::MongoDBConnection;
use database::redis::RedisConnection;

pub async fn run(interaction: Box<ApplicationCommand>, mongodb: MongoDBConnection, _: RedisConnection, discord_http: Arc<Client>) -> Result<CallbackData, String> {

    let option_value = interaction.data.options.first().ok_or("Cannot find `data.options.first`")?.clone().value;
    let option_value = match option_value {
        CommandOptionValue::SubCommand(value) => value,
        _ => return Err("Invalid command: `option_value` is not subcommand".to_string())
    };

    let case_id = option_value.first().ok_or("There is no case_id")?.clone();
    let case_id = match case_id.value {
        CommandOptionValue::Integer(value) => value,
        _ => return Err("Invalid case_id type".to_string())
    };

    let mut case = mongodb.cases.find_one(
        doc! { "index": case_id, "removed": false }, None
    ).await.map_err(|err| format!("{:?}", err))?.ok_or("There is no case with selected id".to_string())?;

    let member_id = interaction.member.ok_or("Cannot get member data".to_string())?
        .user.ok_or("Cannot get user data".to_string())?.id;

    if case.moderator_id != member_id {
        return Err("You can't edit cases created by someone else".to_string())
    }

    let reason = option_value.last().ok_or("There is no reason option")?.clone();
    let reason = match reason.value {
        CommandOptionValue::String(value) => value,
        _ => return Err("Invalid case_id type".to_string())
    };

    mongodb.cases.update_one(
        doc! { "index": case_id, "removed": false }, doc! { "$set": {"reason": reason.clone() } }, None
    ).await.map_err(|err| format!("{:?}", err))?;

    case.reason = Some(reason);

    Ok(CallbackData {
        allowed_mentions: None,
        components: None,
        content: Some("**Case updated**".to_string()),
        embeds: Some(vec![case.to_embed(discord_http).await?]),
        flags: Some(MessageFlags::EPHEMERAL),
        tts: None
    })

}
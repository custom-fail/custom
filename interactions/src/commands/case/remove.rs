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
    }.first().ok_or("There is no `option_value.value`")?.clone();

    let case_id = match option_value.value {
        CommandOptionValue::Integer(value) => value,
        _ => return Err("Invalid case_id type".to_string())
    };

    let removed_case = mongodb.cases.find_one_and_update(
        doc! { "index": case_id, "removed": false }, doc! { "$set": {"removed": true } }, None
    ).await.map_err(|err| format!("{:?}", err))?.ok_or("Cannot find case with selected id")?;

    Ok(CallbackData {
        allowed_mentions: None,
        components: None,
        content: Some("**Removed case**".to_string()),
        embeds: Some(vec![removed_case.to_embed(discord_http).await?]),
        flags: Some(MessageFlags::EPHEMERAL),
        tts: None
    })

}
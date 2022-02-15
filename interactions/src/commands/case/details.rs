use std::sync::Arc;
use mongodb::bson::doc;
use twilight_http::Client;
use twilight_model::application::callback::CallbackData;
use twilight_model::application::interaction::application_command::CommandOptionValue;
use twilight_model::application::interaction::ApplicationCommand;
use database::mongodb::MongoDBConnection;
use database::redis::RedisConnection;
use crate::utilities::embed::embed_to_response;

pub async fn run(interaction: Box<ApplicationCommand>, mongodb: MongoDBConnection, _: RedisConnection, discord_http: Arc<Client>) -> Result<CallbackData, String> {

    let guild_id = interaction.guild_id.ok_or("Cannot find guild_id".to_string())?;

    let option_value = interaction.data.options.first().ok_or("Cannot find `data.options.first`")?.clone().value;
    let option_value = match option_value {
        CommandOptionValue::SubCommand(value) => value,
        _ => return Err("Invalid command: `option_value` is not subcommand".to_string())
    }.first().ok_or("There is no `option_value.value`")?.clone();

    let case_id = match option_value.value {
        CommandOptionValue::Integer(value) => value,
        _ => return Err("Invalid case_id type".to_string())
    };

    let case = mongodb.cases.find_one(
        doc! { "guild_id": guild_id.to_string(), "index": case_id, "removed": false }, None
    ).await.map_err(|err| format!("{err}"))?.ok_or("Cannot find case with selected id".to_string())?;

    Ok(embed_to_response(case.to_embed(discord_http).await?))

}
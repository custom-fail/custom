use std::sync::Arc;
use mongodb::bson::doc;
use twilight_http::Client;
use twilight_model::application::callback::CallbackData;
use twilight_model::application::interaction::application_command::CommandOptionValue;
use database::mongodb::MongoDBConnection;
use database::redis::RedisConnection;
use crate::check_type;
use crate::commands::context::CommandContext;
use crate::utilities::embed::embed_to_response;

pub async fn run(interaction: CommandContext, mongodb: MongoDBConnection, _: RedisConnection, discord_http: Arc<Client>) -> Result<CallbackData, String> {

    let guild_id = interaction.guild_id.ok_or("Cannot find guild_id".to_string())?;
    let case_id = check_type!(
        interaction.options.get("id").ok_or("There is no case id".to_string())?,
        CommandOptionValue::Integer
    ).ok_or("Case id type not match".to_string())?.clone();

    let case = mongodb.cases.find_one(
        doc! { "guild_id": guild_id.to_string(), "index": case_id, "removed": false }, None
    ).await.map_err(|err| format!("{err}"))?.ok_or("Cannot find case with selected id".to_string())?;

    Ok(embed_to_response(case.to_embed(discord_http).await?))

}
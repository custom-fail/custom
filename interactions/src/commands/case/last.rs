use std::sync::Arc;
use mongodb::bson::doc;
use mongodb::options::FindOneOptions;
use twilight_http::Client;
use twilight_model::application::callback::CallbackData;
use twilight_model::application::interaction::ApplicationCommand;
use database::mongodb::MongoDBConnection;
use database::redis::RedisConnection;
use crate::utilities::embed::embed_to_response;

pub async fn run(interaction: Box<ApplicationCommand>, mongodb: MongoDBConnection, _: RedisConnection, discord_http: Arc<Client>) -> Result<CallbackData, String> {

    let guild_id = interaction.guild_id.ok_or("Cannot find guild_id".to_string())?;

    let (member_id, _) = Vec::from_iter(interaction.data.resolved.ok_or("No member specified".to_string())?
        .members.into_iter()).first().cloned().ok_or("Cannot find member information")?;

    let case = mongodb.cases.find_one(
        doc! { "guild_id": guild_id.to_string(), "member_id": member_id.to_string(), "removed": false },
        FindOneOptions::builder().sort(doc! { "created_at": -1 }).build()
    ).await.map_err(|err| format!("{:?}", err))?.ok_or("This user has no cases".to_string())?;

    Ok(embed_to_response(case.to_embed(discord_http).await?))

}
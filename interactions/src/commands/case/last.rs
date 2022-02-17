use std::sync::Arc;
use mongodb::bson::doc;
use mongodb::options::FindOneOptions;
use twilight_http::Client;
use twilight_model::application::callback::CallbackData;
use twilight_model::application::interaction::application_command::CommandOptionValue;
use twilight_model::application::interaction::ApplicationCommand;
use twilight_model::id::Id;
use twilight_model::id::marker::UserMarker;
use database::mongodb::MongoDBConnection;
use database::redis::RedisConnection;
use crate::check_type;
use crate::commands::case::get_member_from_command_data;
use crate::commands::context::InteractionContext;
use crate::utilities::embed::embed_to_response;

pub async fn run(interaction: InteractionContext, mongodb: MongoDBConnection, _: RedisConnection, discord_http: Arc<Client>) -> Result<CallbackData, String> {

    let guild_id = interaction.guild_id.ok_or("Cannot find guild_id".to_string())?;
    let member_id = check_type!(
        interaction.options.get("member").ok_or("There is no member id".to_string())?,
        CommandOptionValue::User
    ).ok_or("Member id type not match".to_string())?.clone();

    let case = mongodb.cases.find_one(
        doc! { "guild_id": guild_id.to_string(), "member_id": member_id.to_string(), "removed": false },
        FindOneOptions::builder().sort(doc! { "created_at": (-1 as i32) }).build()
    ).await.map_err(|err| format!("{err}"))?.ok_or("This user has no cases".to_string())?;

    Ok(embed_to_response(case.to_embed(discord_http).await?))

}
use std::sync::Arc;
use mongodb::bson::doc;
use twilight_http::Client;
use twilight_model::application::interaction::application_command::CommandOptionValue;
use database::models::config::GuildConfig;
use database::mongodb::MongoDBConnection;
use database::redis::RedisConnection;
use utils::embeds::interaction_response_data_from_embed;
use utils::errors::Error;
use crate::commands::context::InteractionContext;
use crate::commands::ResponseData;
use crate::{extract, get_required_option, get_option};

pub async fn run(
    context: InteractionContext,
    mongodb: MongoDBConnection,
    _: RedisConnection,
    discord_http: Arc<Client>,
    _: GuildConfig
) -> ResponseData {
    extract!(context.interaction, guild_id);

    let case_id = get_required_option!(
        context.options.get("id"), CommandOptionValue::Integer
    );

    let case = mongodb.cases.find_one(
        doc! {
            "guild_id": guild_id.to_string(),
            "index": case_id,
            "removed": false
        }, None
    ).await.map_err(Error::from)?.ok_or("Cannot find case with selected id")?;

    Ok((interaction_response_data_from_embed(
        case.to_embed(discord_http).await?,
        false
    ), None))
}
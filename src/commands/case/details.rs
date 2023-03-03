use std::sync::Arc;
use mongodb::bson::doc;
use twilight_http::Client;
use twilight_model::application::interaction::application_command::CommandOptionValue;
use crate::commands::context::InteractionContext;
use crate::commands::ResponseData;
use crate::context::Context;
use crate::{extract, get_required_option, get_option, MongoDBConnection, RedisConnection};
use crate::models::config::GuildConfig;
use crate::utils::embeds::interaction_response_data_from_embed;
use crate::utils::errors::Error;

pub async fn run(
    interaction: InteractionContext,
    context: Arc<Context>,
    discord_http: Arc<Client>,
    _: GuildConfig
) -> ResponseData {
    extract!(interaction.orginal, guild_id);

    let case_index = get_required_option!(
            interaction.options.get("number"), CommandOptionValue::Integer
    );

    let case = context.mongodb.cases.find_one(
        doc! {
            "guild_id": guild_id.to_string(),
            "index": case_index,
            "removed": false
        }, None
    ).await.map_err(Error::from)?.ok_or("Cannot find case with selected id")?;

    Ok((interaction_response_data_from_embed(
        case.to_embed(discord_http).await?,
        false
    ), None))
}
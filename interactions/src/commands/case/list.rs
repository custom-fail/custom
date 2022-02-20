use crate::utilities::embed::embed_from_fields;
use database::mongodb::MongoDBConnection;
use database::redis::RedisConnection;
use mongodb::bson::doc;
use mongodb::options::FindOptions;
use std::sync::Arc;
use futures::TryStreamExt;
use twilight_http::Client;
use twilight_model::application::callback::CallbackData;
use twilight_model::application::component::{ActionRow, Component, SelectMenu};
use twilight_model::application::component::select_menu::SelectMenuOption;
use twilight_model::application::interaction::application_command::CommandOptionValue;
use database::models::case::Case;
use crate::check_type;
use crate::commands::context::InteractionContext;

pub async fn run(
    interaction: InteractionContext,
    mongodb: MongoDBConnection,
    _: RedisConnection,
    _: Arc<Client>,
) -> Result<CallbackData, String> {

    let user_id = interaction.user.ok_or("There is no user information")?.id;
    let guild_id = interaction.guild_id.ok_or("Cannot find guild_id".to_string())?;

    let member_id = check_type!(
        interaction.options.get("member").ok_or("There is no member id".to_string())?,
        CommandOptionValue::User
    ).ok_or("Member id type not match".to_string())?.clone();

    let page = u64::try_from(
        check_type!(
            interaction.options.get("page").unwrap_or(&CommandOptionValue::Integer(1)),
            CommandOptionValue::Integer
        ).ok_or("Case id type not match".to_string())?.clone()
    ).map_err(|_| "Page must be u64".to_string())?;

    let case_list = mongodb.cases.find(
        doc! { "member_id": member_id.to_string(), "guild_id": guild_id.to_string(), "removed": false },
        FindOptions::builder().limit(6).skip(Some(page - 1)).sort(doc! { "created_at": -1 as i32 }).build()
    ).await.map_err(|err| format!("{:?}", err))?;

    let count = mongodb.cases.count_documents(
        doc! { "member_id": member_id.to_string(), "guild_id": guild_id.to_string(), "removed": false }, None
    ).await.map_err(|err| format!("{err}"))?;

    let case_list: Vec<Case> = case_list.try_collect().await.map_err(|err| format!("{err}"))?;

    if case_list.len() < 1 {
        return Err("This user has no cases".to_string())
    }

    let fields = case_list.into_iter().map(|case| case.to_field()).collect();
    let embed = embed_from_fields(fields);

    let pages  = if count % 6 == 0 { count / 6 } else { count / 6 + 1 };

    let mut result = vec![];
    for page in 1..(pages + 1) {
        result.push(SelectMenuOption {
            default: false,
            description: None,
            emoji: None,
            label: format!("Page {page}"),
            value: page.to_string()
        });
    }

    Ok(CallbackData {
        allowed_mentions: None,
        components: Some(vec![
            Component::ActionRow(ActionRow {
                components: vec![
                    Component::SelectMenu(SelectMenu {
                        custom_id: format!("a:{user_id}:cl:{member_id}"),
                        disabled: false,
                        max_values: Some(1),
                        min_values: Some(1),
                        options: result,
                        placeholder: None
                    })
                ]
            })
        ]),
        content: None,
        embeds: Some(vec![embed]),
        flags: None,
        tts: None
    })

}

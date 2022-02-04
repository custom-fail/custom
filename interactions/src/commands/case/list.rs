use crate::commands::case::get_member_from_command_data;
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
use twilight_model::application::interaction::ApplicationCommand;
use database::models::case::Case;

pub async fn run(
    interaction: Box<ApplicationCommand>,
    mongodb: MongoDBConnection,
    _: RedisConnection,
    _: Arc<Client>,
) -> Result<CallbackData, String> {

    let user_id = interaction.clone().member.ok_or("Unknown member (DM invoked command)".to_string())?
        .user.ok_or("Unknown user".to_string())?.id;

    let guild_id = interaction
        .guild_id
        .ok_or("Cannot find guild_id".to_string())?;
    let (member_id, _) = get_member_from_command_data(interaction)?;

    let case_list = mongodb.cases.find(
        doc! { "member_id": member_id.to_string(), "guild_id": guild_id.to_string(), "removed": false },
        FindOptions::builder().limit(6).sort(doc! { "created_at": -1 as i32 }).build()
    ).await.map_err(|err| format!("{:?}", err))?;

    let count = mongodb.cases.count_documents(
        doc! { "member_id": member_id.to_string(), "guild_id": guild_id.to_string(), "removed": false }, None
    ).await.map_err(|e| format!("{:?}", e))?;

    let case_list: Vec<Case> = case_list.try_collect().await.map_err(|e| format!("{:?}", e))?;

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
            label: format!("Page {}", page),
            value: page.to_string()
        });
    }

    Ok(CallbackData {
        allowed_mentions: None,
        components: Some(vec![
            Component::ActionRow(ActionRow {
                components: vec![
                    Component::SelectMenu(SelectMenu {
                        custom_id: format!("{}:{}:case", user_id.to_string() ,member_id.to_string()),
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

use database::mongodb::MongoDBConnection;
use database::redis::RedisConnection;
use mongodb::bson::doc;
use mongodb::options::FindOptions;
use std::sync::Arc;
use futures::{StreamExt, TryStreamExt};
use mongodb::bson;
use twilight_http::Client;
use twilight_model::http::interaction::InteractionResponseData;
use twilight_model::application::component::{ActionRow, Component, SelectMenu};
use twilight_model::application::component::select_menu::SelectMenuOption;
use twilight_model::application::interaction::application_command::CommandOptionValue;
use database::models::case::Case;
use database::models::config::GuildConfig;
use utils::check_type;
use utils::errors::Error;
use serde::{Serialize, Deserialize};
use twilight_model::channel::embed::{Embed, EmbedAuthor, EmbedFooter};
use utils::avatars::get_avatar_url;
use crate::commands::context::InteractionContext;
use crate::commands::ResponseData;

#[derive(Serialize, Deserialize)]
struct ActionDocument {
    action: u8
}

#[derive(Serialize, Deserialize)]
struct CountActions {
    #[serde(rename = "_id")]
    target: ActionDocument,
    count: usize
}

pub async fn run(
    interaction: InteractionContext,
    mongodb: MongoDBConnection,
    _: RedisConnection,
    _: Arc<Client>,
    _: GuildConfig
) -> ResponseData {
    let user_id = interaction.user.ok_or("There is no user information")?.id;
    let guild_id = interaction.guild_id.ok_or("Cannot find guild_id")?;

    let member_id = *check_type!(
        interaction.options.get("member").ok_or("There is no member id")?,
        CommandOptionValue::User
    ).ok_or("Member id type not match")?;

    let page = u64::try_from(
        *check_type!(
            interaction.options.get("page").unwrap_or(&CommandOptionValue::Integer(1)),
            CommandOptionValue::Integer
        ).ok_or("Case id type not match")?
    ).map_err(|_| "Page must be u64")?;

    let action_type = match interaction.options.get("type") {
        Some(CommandOptionValue::String(value)) => {
            Some(match value.as_str() {
                "mutes" => 7,
                "warns" => 1,
                "bans" => 4,
                "kicks" => 6,
                _ => 0
            })
        },
        _ => None
    };

    let filter = if let Some(action_type) = action_type {
        doc! { "member_id": member_id.to_string(), "guild_id": guild_id.to_string(), "removed": false, "action": (action_type as i64) }
    } else {
        doc! { "member_id": member_id.to_string(), "guild_id": guild_id.to_string(), "removed": false }
    };

    let case_list = mongodb.cases.find(
        filter.clone(),
        FindOptions::builder()
            .limit(6).skip(Some((page - 1) * 6))
            .sort(doc! { "created_at": -1_i32 }).build()
    ).await.map_err(Error::from)?;

    let case_list: Vec<Case> = case_list.try_collect().await.map_err(Error::from)?;

    if case_list.is_empty() {
        return Err(Error::from("This user has no cases"))
    }

    let mut count = mongodb.cases.aggregate(
        [
            doc! { "$match": filter },
            doc! {
                "$group": {
                    "_id": { "action": "$action" },
                    "count": { "$sum": 1_u32 },
                    "totalValue": { "$sum": "$count" }
                }
            }
        ],
        None
    ).await.map_err(Error::from)?;

    let mut total = 0;
    let mut footer = vec![];

    while let Some(result) = count.next().await {

        let result = result.map_err(Error::from)?;
        let result: CountActions = bson::from_document(result).map_err(Error::from)?;

        if let Some(action_type) = action_type {
            if action_type == result.target.action {
                total += result.count;
            }
        } else { total += result.count }

        footer.push(format!("{}: {}", match result.target.action {
            7 => "Mutes",
            1 => "Warns",
            4 => "Bans",
            6 => "Kicks",
            _ => "???"
        }, result.count));

    }

    let author = if let Some(user) = interaction.resolved.users.get(&member_id) {
        let avatar = get_avatar_url(user.avatar, user.id);
        EmbedAuthor {
            icon_url: Some(avatar.to_owned()),
            name: format!("{}#{} {}", user.name, user.discriminator, user_id),
            proxy_icon_url: Some(avatar),
            url: None
        }
    } else {
        EmbedAuthor {
            icon_url: Some("https://cdn.discordapp.com/embed/avatars/0.png".to_string()),
            name: format!("Deleted User#0000 {member_id}"),
            proxy_icon_url: Some("https://cdn.discordapp.com/embed/avatars/0.png".to_string()),
            url: None
        }
    };

    let fields = case_list.into_iter().map(|case| case.to_field()).collect();
    let embed = Embed {
        author: Some(author),
        color: None,
        description: None,
        fields,
        footer: Some(EmbedFooter {
            icon_url: None,
            proxy_icon_url: None,
            text: footer.join(" | ")
        }),
        image: None,
        kind: "".to_string(),
        provider: None,
        thumbnail: None,
        timestamp: None,
        title: None,
        url: None,
        video: None
    };

    let pages = if total % 6 == 0 { total / 6 } else { total / 6 + 1 };

    let mut result = vec![];
    for page in 1..(
        if pages + 1 > 25 { 25 } else { pages + 1 }
    ) {
        result.push(SelectMenuOption {
            default: false,
            description: None,
            emoji: None,
            label: format!("Page {page}"),
            value: page.to_string()
        });
    }

    Ok((InteractionResponseData {
        allowed_mentions: None,
        attachments: None,
        choices: None,
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
        custom_id: None,
        embeds: Some(vec![embed]),
        flags: None,
        title: None,
        tts: None
    }, None))

}
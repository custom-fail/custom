use mongodb::bson::doc;
use mongodb::options::FindOptions;
use std::sync::Arc;
use futures_util::{TryStreamExt, StreamExt};
use mongodb::bson;
use twilight_http::Client;
use twilight_model::http::interaction::InteractionResponseData;
use twilight_model::application::component::{ActionRow, Component, SelectMenu};
use twilight_model::application::component::select_menu::SelectMenuOption;
use twilight_model::application::interaction::application_command::CommandOptionValue;
use serde::{Serialize, Deserialize};
use twilight_model::channel::embed::{Embed, EmbedAuthor, EmbedFooter};
use crate::commands::context::{InteractionContext, InteractionHelpers};
use crate::commands::ResponseData;
use crate::context::Context;
use crate::{extract, get_option, get_required_option};
use crate::models::case::Case;
use crate::models::config::GuildConfig;
use crate::utils::avatars::get_avatar_url;
use crate::utils::errors::Error;

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
    context: Arc<Context>,
    _: Arc<Client>,
    _: GuildConfig
) -> ResponseData {
    let member_id = get_required_option!(
        interaction.options.get("member"), CommandOptionValue::User
    );

    let page = u64::try_from(
        get_option!(
            interaction.options.get("page"), CommandOptionValue::Integer
        ).copied().unwrap_or(1)
    ).map_err(|_| "Page must be u64")?;

    let user_data = interaction.orginal.resolved()
        .and_then(|resolved| resolved.users.get(member_id)).cloned();

    extract!(interaction.orginal, guild_id, member);
    extract!(member, user);

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
        doc! {
            "member_id": member_id.to_string(),
            "guild_id": guild_id.to_string(),
            "removed": false,
            "action": (action_type as i64)
        }
    } else {
        doc! {
            "member_id": member_id.to_string(),
            "guild_id": guild_id.to_string(),
            "removed": false
        }
    };

    let case_list = context.mongodb.cases.find(
        filter.clone(),
        FindOptions::builder()
            .limit(6).skip(Some((page - 1) * 6))
            .sort(doc! { "created_at": -1_i32 }).build()
    ).await.map_err(Error::from)?;

    let case_list: Vec<Case> = case_list.try_collect().await.map_err(Error::from)?;

    if case_list.is_empty() {
        return Err(Error::from("This user has no cases"))
    }

    let mut count = context.mongodb.cases.aggregate(
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

    let author = if let Some(user) = user_data {
        let avatar = get_avatar_url(user.avatar, user.id);
        EmbedAuthor {
            icon_url: Some(avatar.to_owned()),
            name: format!("{}#{} {}", user.name, user.discriminator, user.id),
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
                        custom_id: format!("a:{}:cl:{member_id}", user.id),
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
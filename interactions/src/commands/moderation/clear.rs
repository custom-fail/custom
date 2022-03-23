use std::sync::Arc;
use twilight_http::Client;
use twilight_model::application::interaction::application_command::CommandOptionValue;
use twilight_model::channel::Message;
use twilight_model::channel::message::{MessageFlags, MessageType};
use twilight_model::http::interaction::InteractionResponseData;
use twilight_model::id::Id;
use twilight_model::id::marker::MessageMarker;
use database::models::config::GuildConfig;
use database::mongodb::MongoDBConnection;
use database::redis::RedisConnection;
use utils::check_type;
use crate::commands::ResponseData;
use crate::InteractionContext;

pub async fn run(interaction: InteractionContext, _: MongoDBConnection, _: RedisConnection, discord_http: Arc<Client>, _: GuildConfig) -> ResponseData {

    let channel_id = interaction.channel_id;
    let amount = check_type!(
        interaction.options.get("amount").ok_or("There is no amount value".to_string())?,
        CommandOptionValue::Integer
    ).ok_or("".to_string())?;

    let member = match interaction.options.get("member") {
        Some(member) => check_type!(member, CommandOptionValue::User),
        None => None
    }.cloned();

    let filter = match interaction.options.get("filter") {
        Some(filter) => check_type!(filter, CommandOptionValue::String),
        None => None
    }.cloned();

    if amount < &2 || amount > &600 {
        return Err("You can clear up to 600 messages".to_string())
    }

    let loops = if amount % 100 != 0 { (amount / 100) + 1 } else { amount / 50 };
    let mut last = None;

    for i in 0..loops {

        let amount = if loops - i == 1 { amount - (i * 100) } else { 100 };

        let mut messages = if let Some(last) = last {
            discord_http.channel_messages(channel_id)
                .limit(amount as u16).map_err(|err| err.to_string())?
                .after(last)
                .exec().await.map_err(|err| err.to_string())?
                .model().await.map_err(|err| err.to_string())?
        } else {
            discord_http.channel_messages(channel_id)
                .limit(amount as u16).map_err(|err| err.to_string())?
                .exec().await.map_err(|err| err.to_string())?
                .model().await.map_err(|err| err.to_string())?
        };

        last = messages.last().map(|msg| msg.id);

        if let Some(member) = member {
            messages = messages.iter()
                .filter(|msg| msg.author.id == member)
                .cloned().collect::<Vec<Message>>();
        }

        if let Some(filter) = &filter {
            messages = messages.iter()
                .filter(|msg| {
                    match filter.as_str() {
                        "system" => msg.kind != MessageType::Regular,
                        "attachments" => !msg.attachments.is_empty(),
                        "stickers" => !msg.sticker_items.is_empty(),
                        "embeds" => !msg.embeds.is_empty(),
                        _ /*bots*/ => msg.author.bot
                    }
                }).cloned().collect::<Vec<Message>>();
        }

        let messages = messages.iter()
            .map(|message| message.id).collect::<Vec<Id<MessageMarker>>>();

        if messages.len() < 1 { continue }

        discord_http.delete_messages(channel_id, &messages)
            .exec().await.map_err(|err| err.to_string())?;

    }

    Ok((InteractionResponseData {
        allowed_mentions: None,
        attachments: None,
        choices: None,
        components: None,
        content: Some("Deleting messages".to_string()),
        custom_id: None,
        embeds: None,
        flags: Some(MessageFlags::EPHEMERAL),
        title: None,
        tts: None
    }, None))

}
use crate::commands::context::InteractionContext;
use crate::commands::ResponseData;
use crate::context::Context;
use crate::models::config::GuildConfig;
use crate::utils::errors::Error;
use crate::{extract, get_option, get_required_option, render_context};
use std::sync::Arc;
use twilight_http::Client;
use twilight_model::application::interaction::application_command::CommandOptionValue;
use twilight_model::channel::message::MessageType;
use twilight_model::channel::Message;
use twilight_model::id::marker::MessageMarker;
use twilight_model::id::Id;

pub async fn run(
    interaction: InteractionContext,
    context: Arc<Context>,
    discord_http: Arc<Client>,
    config: GuildConfig,
) -> ResponseData {
    extract!(interaction.orginal, channel_id);

    let amount = get_required_option!(
        interaction.options.get("amount"),
        CommandOptionValue::Integer
    );

    let member = get_option!(interaction.options.get("member"), CommandOptionValue::User).copied();

    let filter = get_option!(
        interaction.options.get("member"),
        CommandOptionValue::String
    );

    if !(&2..=&600).contains(&amount) {
        return Err(Error::from("You can clear up to 600 messages"));
    }

    let loops = if amount % 100 != 0 {
        (amount / 100) + 1
    } else {
        amount / 50
    };
    let mut last = None;

    let mut total = 0;

    for i in 0..loops {
        let amount = if loops - i == 1 {
            amount - (i * 100)
        } else {
            100
        };

        let mut messages = if let Some(last) = last {
            discord_http
                .channel_messages(channel_id)
                .limit(amount as u16)
                .map_err(Error::from)?
                .after(last)
                .await
                .map_err(Error::from)?
                .model()
                .await
                .map_err(Error::from)?
        } else {
            discord_http
                .channel_messages(channel_id)
                .limit(amount as u16)
                .map_err(Error::from)?
                .await
                .map_err(Error::from)?
                .model()
                .await
                .map_err(Error::from)?
        };

        last = messages.last().map(|msg| msg.id);

        if let Some(member) = member {
            messages = messages
                .iter()
                .filter(|msg| msg.author.id == member)
                .cloned()
                .collect::<Vec<Message>>();
        }

        if let Some(filter) = filter {
            messages = messages
                .iter()
                .filter(|msg| {
                    match filter.as_str() {
                        "system" => msg.kind != MessageType::Regular,
                        "attachments" => !msg.attachments.is_empty(),
                        "stickers" => !msg.sticker_items.is_empty(),
                        "embeds" => !msg.embeds.is_empty(),
                        _ /*bots*/ => msg.author.bot
                    }
                })
                .cloned()
                .collect::<Vec<Message>>();
        }

        let messages = messages
            .iter()
            .map(|message| message.id)
            .collect::<Vec<Id<MessageMarker>>>();

        if messages.is_empty() {
            continue;
        }
        total += messages.len();

        discord_http
            .delete_messages(channel_id, &messages)
            .map_err(Error::from)?
            .await
            .map_err(Error::from)?;
    }

    Ok((
        config
            .assets
            .render_message(
                &context.assets,
                "commands.clear",
                &mut render_context!(
                    ["interaction", &interaction.orginal],
                    ["amount", &amount],
                    ["cleared", &total]
                ),
                &context.redis,
            )
            .await?,
        None,
    ))
}

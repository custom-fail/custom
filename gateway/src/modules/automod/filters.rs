use twilight_model::channel::Message;
use database::models::config::automod::filters::{Attachments, Filters, MessageLength};

pub fn filters_match(filter: &Filters, message: &Message) -> bool {
    match filter {
        Filters::Attachments(config) => attachments(config, message),
        Filters::MessageLength(config) => message_length(config, message),
        Filters::Stickers => stickers(message)
    }
}

fn attachments(config: &Attachments, message: &Message) -> bool {
    min_max(
        config.min.map(usize::from),
        config.max.map(usize::from),
        message.attachments.len()
    )
}

fn message_length(config: &MessageLength, message: &Message) -> bool {
    min_max(
        config.min.map(usize::from),
        config.max.map(usize::from),
        message.content.len()
    )
}

fn stickers(message: &Message) -> bool { !message.sticker_items.is_empty() }

fn min_max(min: Option<usize>, max: Option<usize>, count: usize) -> bool {
    (if let Some(min) = min {
        (min as usize) > count
    } else { false }) || (if let Some(max) = max {
        (max as usize) < count
    } else { false })
}
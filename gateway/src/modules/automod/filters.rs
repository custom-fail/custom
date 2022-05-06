use twilight_model::channel::Message;
use database::models::config::automod::filters::{Attachments, Filters, MessageLength};

pub fn filters_match(filter: Filters, message: Message) -> bool {
    match filter {
        Filters::Attachments(config) => attachments(config, message),
        Filters::MessageLength(config) => message_length(config, message),
        Filters::Stickers => stickers(message)
    }
}

fn attachments(config: Attachments, message: Message) -> bool {
    let message_attachments_count = message.attachments.len();
    (if let Some(min) = config.min {
        (min as usize) > message_attachments_count
    } else { false }) || (if let Some(max) = config.max {
        (max as usize) < message_attachments_count
    } else { false })
}

fn message_length(config: MessageLength, message: Message) -> bool {
    let message_content_len = message.content.len();
    (if let Some(min) = config.min {
        (min as usize) > message_content_len
    } else { false }) || (if let Some(max) = config.max {
        (max as usize) < message_content_len
    } else { false })
}

fn stickers(message: Message) -> bool { !message.sticker_items.is_empty() }
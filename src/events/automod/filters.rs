use twilight_model::channel::Message;
use crate::models::config::automod::filters::{Attachments, Filter, MessageLength};

impl Filter {
    pub fn is_matching(&self, message: &Message) -> bool {
        match &self {
            Filter::Attachments(config) => Self::attachments(config, message),
            Filter::MessageLength(config) => Self::message_length(config, message),
            Filter::Stickers => Self::stickers(message)
        }
    }

    fn attachments(config: &Attachments, message: &Message) -> bool {
        let message_attachments_count = message.attachments.len();
        (if let Some(min) = config.min {
            (min as usize) > message_attachments_count
        } else { false }) || (if let Some(max) = config.max {
            (max as usize) < message_attachments_count
        } else { false })
    }

    fn message_length(config: &MessageLength, message: &Message) -> bool {
        let message_content_len = message.content.len();
        (if let Some(min) = config.min {
            (min as usize) > message_content_len
        } else { false }) || (if let Some(max) = config.max {
            (max as usize) < message_content_len
        } else { false })
    }

    fn stickers(message: &Message) -> bool { !message.sticker_items.is_empty() }
}


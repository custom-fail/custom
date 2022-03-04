use twilight_model::channel::embed::{Embed, EmbedField};
use twilight_model::http::interaction::InteractionResponseData;

fn text_to_embed(title: String, description: String) -> Embed {
    Embed {
        author: None,
        color: None,
        description: Some(description),
        fields: vec![],
        footer: None,
        image: None,
        kind: "".to_string(),
        provider: None,
        thumbnail: None,
        timestamp: None,
        title: Some(title),
        url: None,
        video: None
    }
}

pub fn text_to_response_embed(title: String, description: String) -> InteractionResponseData {
    InteractionResponseData {
        allowed_mentions: None,
        attachments: None,
        choices: None,
        components: None,
        content: None,
        custom_id: None,
        embeds: Some(vec![text_to_embed(title, description)]),
        flags: None,
        title: None,
        tts: None
    }
}

pub fn embed_to_response(embed: Embed) -> InteractionResponseData {
    InteractionResponseData {
        allowed_mentions: None,
        attachments: None,
        choices: None,
        components: None,
        content: None,
        custom_id: None,
        embeds: Some(vec![embed]),
        flags: None,
        title: None,
        tts: None
    }
}

pub fn embed_from_fields(fields: Vec<EmbedField>) -> Embed {
    Embed {
        author: None,
        color: None,
        description: None,
        fields,
        footer: None,
        image: None,
        kind: "".to_string(),
        provider: None,
        thumbnail: None,
        timestamp: None,
        title: None,
        url: None,
        video: None
    }
}
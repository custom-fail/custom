use twilight_model::application::callback::CallbackData;
use twilight_model::channel::embed::{Embed, EmbedField};

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

pub fn text_to_response_embed(title: String, description: String) -> CallbackData {
    CallbackData {
        allowed_mentions: None,
        components: None,
        content: None,
        embeds: Some(vec![text_to_embed(title, description)]),
        flags: None,
        tts: None
    }
}

pub fn embed_to_response(embed: Embed) -> CallbackData {
    CallbackData {
        allowed_mentions: None,
        components: None,
        content: None,
        embeds: Some(vec![embed]),
        flags: None,
        tts: None
    }
}

pub fn response_from_embed_fields(fields: Vec<EmbedField>) -> CallbackData {
    embed_to_response(Embed {
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
    })
}
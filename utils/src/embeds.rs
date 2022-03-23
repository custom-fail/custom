use twilight_model::channel::embed::{Embed, EmbedField};
use twilight_model::channel::message::MessageFlags;
use twilight_model::http::interaction::InteractionResponseData;

pub struct EmbedBuilder {
    description: Option<String>,
    title: Option<String>,
    fields: Vec<EmbedField>
}

impl EmbedBuilder {
    pub fn new() -> Self {
        Self {
            title: None,
            description: None,
            fields: vec![]
        }
    }

    pub fn description(&self, text: String) -> Self {
        Self {
            title: self.title.to_owned(),
            description: Some(text),
            fields: self.fields.to_owned()
        }
    }

    pub fn title(&self, text: String) -> Self {
        Self {
            title: Some(text),
            description: None,
            fields: self.fields.to_owned()
        }
    }

    pub fn fields(&self, fields: Vec<EmbedField>) -> Self {
        Self {
            description: self.description.to_owned(),
            title: self.title.to_owned(),
            fields
        }
    }

    pub fn to_embed(&self) -> Embed {
        Embed {
            author: None,
            color: None,
            description: self.description.to_owned(),
            fields: self.fields.to_owned(),
            footer: None,
            image: None,
            kind: "".to_string(),
            provider: None,
            thumbnail: None,
            timestamp: None,
            title: self.title.to_owned(),
            url: None,
            video: None
        }
    }

    pub fn to_interaction_response_data(&self, ephemeral: bool) -> InteractionResponseData {
        interaction_response_data_from_embed(self.to_embed(), ephemeral)
    }
}

pub fn interaction_response_data_from_embed(embed: Embed, ephemeral: bool) -> InteractionResponseData {
    InteractionResponseData {
        allowed_mentions: None,
        attachments: None,
        choices: None,
        components: None,
        content: None,
        custom_id: None,
        embeds: Some(vec![embed]),
        flags: if ephemeral { Some(MessageFlags::EPHEMERAL) } else { None },
        title: None,
        tts: None
    }
}
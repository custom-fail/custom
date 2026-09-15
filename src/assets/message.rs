use serde::{Deserialize, Serialize};
use twilight_model::channel::message::embed::{EmbedAuthor, EmbedFooter};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum CaseMessageType {
    #[serde(rename = "non-server")]
    NonServer,
    #[serde(rename = "moderation-log")]
    ModerationLog,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub content: Option<String>,
    #[serde(default)]
    pub ephemeral: bool,
    #[serde(default)]
    pub embeds: Vec<Embed>,
    #[serde(default, rename = "add-case")]
    pub add_case: Option<CaseMessageType>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Embed {
    pub title: Option<String>,
    pub description: Option<String>,
    pub thumbnail: Option<String>,
    pub footer: Option<TextIcon>,
    #[serde(default)]
    pub fields: Vec<EmbedField>,
    pub author: Option<TextIcon>,
    pub color: Option<u32>,
    pub image: Option<String>,
    pub video: Option<String>,
    pub url: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EmbedField {
    pub inline: Option<String>,
    pub name: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TextIcon {
    pub text: String,
    pub icon_url: Option<String>,
}

impl Into<EmbedAuthor> for TextIcon {
    fn into(self) -> EmbedAuthor {
        EmbedAuthor {
            icon_url: self.icon_url.to_owned(),
            name: self.text,
            proxy_icon_url: self.icon_url,
            url: None,
        }
    }
}

impl Into<EmbedFooter> for TextIcon {
    fn into(self) -> EmbedFooter {
        EmbedFooter {
            icon_url: self.icon_url.to_owned(),
            proxy_icon_url: self.icon_url,
            text: self.text,
        }
    }
}

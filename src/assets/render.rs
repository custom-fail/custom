use std::str::FromStr;

use tera::{Tera, Context, Error as TeraError};
use twilight_model::{http::interaction::InteractionResponseData, channel::message::{MessageFlags, AllowedMentions, embed::{Embed as DiscordEmbed, EmbedVideo, EmbedImage, EmbedThumbnail, EmbedField as DiscordEmbedField}}};

use super::{message::{Message, Embed, EmbedField, TextIcon}, AssetsManager, GuildAssets};

#[derive(Debug)]
pub enum RenderErrorKind {
    Tera(TeraError),
    InvalidMessage,
    BoolConvertion
}

#[derive(Debug)]
pub struct RenderError(RenderErrorKind);

impl GuildAssets {
    pub async fn render_message(
        &self,
        manager: &AssetsManager,
        name: &str,
        data: &Context
    ) -> Result<InteractionResponseData, RenderError> {
        for asset_name in &(self.0) {
            let asset = manager.custom.read().await.get(asset_name).cloned();
            // Option::map won't be used becouse of lifetiems
            if let Some(asset) = asset {
                if let Some(message) = asset.get(name) {
                    return message.render(data)   
                }
            }
        }
        
        let message = manager.default.get(name).ok_or(RenderError(RenderErrorKind::InvalidMessage))?;
        message.render(data)
    }
}

macro_rules! render_option {
    ($target: expr, $data: expr) => {
        if let Some(name) = ($target).as_ref() {
            let content = render_string(name, $data)?;
            if content == "!Skip" { None } else { Some(content) }
        } else { None }
    };
}

macro_rules! render_optional_text_icon {
    ($target: expr, $data: expr) => {
        if let Some(name) = ($target).as_ref() {
            Some(name.render($data)?.into())
        } else { None }
    };
}

impl Message {
    pub fn render(&self, data: &Context) -> Result<InteractionResponseData, RenderError> {
        let mut embeds = vec![];
        for embed in &self.embeds {
            embeds.push(embed.render(&data)?)
        }
        
        Ok(InteractionResponseData {
            allowed_mentions: Some(AllowedMentions {
                parse: vec![],
                replied_user: true,
                roles: vec![],
                users: vec![],
            }),
            attachments: None,
            choices: None,
            components: None,
            content: render_option!(self.content, data),
            custom_id: None,
            embeds: Some(embeds),
            flags: if self.ephemeral { Some(MessageFlags::EPHEMERAL) } else { None },
            title: None,
            tts: Some(false),
        })
    }
}

impl Embed {
    pub fn render(&self, data: &Context) -> Result<DiscordEmbed, RenderError> {
        let mut fields = vec![];
        for field in &self.fields {
            let field = field.render(data)?;
            if &field.name == "!SkipAll" || &field.value == "!SkipAll" { continue }
            fields.push(field);
        }
        
        Ok(DiscordEmbed {
            author: render_optional_text_icon!(self.author, data),
            color: self.color,
            description: render_option!(self.description, data),
            fields,
            footer: render_optional_text_icon!(self.footer, data),
            image: render_option!(self.image, data).map(|image| EmbedImage {
                height: None,
                proxy_url: Some(image.to_owned()),
                url: image,
                width: None,
            }),
            kind: "".to_string(),
            provider: None,
            thumbnail: render_option!(self.thumbnail, data).map(|thumbnail| EmbedThumbnail {
                height: None,
                proxy_url: Some(thumbnail.to_owned()),
                url: thumbnail,
                width: None,
            }),
            timestamp: None,
            title: render_option!(self.title, data),
            url: render_option!(self.url, data),
            video: render_option!(self.video, data).map(|video| EmbedVideo {
                height: None,
                proxy_url: Some(video.to_owned()),
                url: Some(video),
                width: None,
            }),
        })
    }
}

impl TextIcon {
    pub fn render(&self, data: &Context) -> Result<TextIcon, RenderError> {
        Ok(TextIcon {
            text: render_string(&self.text, data)?,
            icon_url: render_option!(self.icon_url, data),
        })
    }
}

impl EmbedField {
    pub fn render(&self, data: &Context) -> Result<DiscordEmbedField, RenderError> {
        Ok(DiscordEmbedField {
            inline: bool::from_str(
                render_option!(self.inline, data).unwrap_or_else(String::new).as_str()
            ).map_err(|_| RenderError(RenderErrorKind::BoolConvertion))?,
            name: render_string(&self.name, data)?,
            value: render_string(&self.value, data)?,
        })
    }
}

fn render_string(content: &String, context: &Context) -> Result<String, RenderError> {
    Tera::one_off(
        content.as_str(), &context, false
    ).map_err(|err| RenderError(RenderErrorKind::Tera(err)))
}

use std::{
    str::{FromStr, ParseBoolError},
    sync::Arc,
};

use tera::{Context, Error as TeraError, Tera};
use twilight_model::{
    channel::message::{
        embed::{
            Embed as DiscordEmbed, EmbedField as DiscordEmbedField, EmbedImage, EmbedThumbnail,
            EmbedVideo,
        },
        AllowedMentions, MessageFlags,
    },
    http::interaction::InteractionResponseData,
};

use crate::database::redis::RedisConnection;

use super::{
    message::{CaseMessageType, Embed, EmbedField, Message, TextIcon},
    AssetsManager, GuildAssets,
};

#[derive(Debug)]
pub enum RenderErrorKind {
    Tera(TeraError),
    InvalidMessage(String),
    BoolConvertion(ParseBoolError),
}

impl ToString for RenderError {
    fn to_string(&self) -> String {
        match &self.0 {
            RenderErrorKind::Tera(err) => {
                format!("{err:#?}")
            }
            RenderErrorKind::InvalidMessage(name) => {
                format!("Cannot find matching message asset for {name}")
            }
            RenderErrorKind::BoolConvertion(output) => {
                format!("{output} while converting to boolen")
            }
        }
    }
}

#[derive(Debug)]
pub struct RenderError(RenderErrorKind);

impl GuildAssets {
    async fn get_message(&self, manager: &AssetsManager, name: &str) -> Option<Arc<Message>> {
        for asset_name in &(self.0) {
            let asset = manager.custom.read().await.get(asset_name).cloned();
            // Option::map won't be used becouse of lifetiems
            if let Some(asset) = asset {
                if let Some(message) = asset.get(name) {
                    return Some(Arc::clone(message));
                }
            }
        }

        manager.default.get(name).map(Arc::clone)
    }

    pub async fn render_message(
        &self,
        manager: &AssetsManager,
        name: &str,
        data: &mut Context,
        redis: &RedisConnection,
    ) -> Result<InteractionResponseData, RenderError> {
        self.get_message(manager, name)
            .await
            .ok_or(RenderError(RenderErrorKind::InvalidMessage(
                name.to_string(),
            )))?
            .render(data, &self, manager, redis)
            .await
    }

    pub async fn render_additional_embed(
        &self,
        manager: &AssetsManager,
        name: &str,
        data: &Context,
    ) -> Result<DiscordEmbed, RenderError> {
        Ok(self
            .get_message(manager, name)
            .await
            .ok_or(RenderError(RenderErrorKind::InvalidMessage(
                name.to_string(),
            )))?
            .embeds
            .get(0)
            .ok_or(RenderError(RenderErrorKind::InvalidMessage(
                name.to_string(),
            )))?
            .render(data)?)
    }
}

macro_rules! render_option {
    ($target: expr, $data: expr) => {
        if let Some(name) = ($target).as_ref() {
            let content = render_string(name, $data)?;
            if content == "!Skip" {
                None
            } else {
                Some(content)
            }
        } else {
            None
        }
    };
}

macro_rules! render_optional_text_icon {
    ($target: expr, $data: expr) => {
        if let Some(name) = ($target).as_ref() {
            Some(name.render($data)?.into())
        } else {
            None
        }
    };
}

impl Message {
    pub async fn render(
        &self,
        data: &mut Context,
        guild_assets: &GuildAssets,
        manager: &AssetsManager,
        redis: &RedisConnection,
    ) -> Result<InteractionResponseData, RenderError> {
        let mut embeds = vec![];
        for embed in &self.embeds {
            embeds.push(embed.render(&data)?)
        }

        if let Some(case_type) = self.add_case {
            let embed = match case_type {
                CaseMessageType::NonServer => {
                    guild_assets
                        .render_additional_embed(manager, "case.non-server", data)
                        .await?
                }
                CaseMessageType::ModerationLog => {
                    guild_assets
                        .render_additional_embed(manager, "case.moderation-log", data)
                        .await?
                }
            };
            embeds.push(embed);
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
            flags: if self.ephemeral {
                Some(MessageFlags::EPHEMERAL)
            } else {
                None
            },
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
            if &field.name == "!SkipAll" || &field.value == "!SkipAll" {
                continue;
            }
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
                render_option!(self.inline, data)
                    .unwrap_or_else(String::new)
                    .as_str(),
            )
            .map_err(|err| RenderError(RenderErrorKind::BoolConvertion(err)))?,
            name: render_string(&self.name, data)?,
            value: render_string(&self.value, data)?,
        })
    }
}

fn render_string(content: &String, context: &Context) -> Result<String, RenderError> {
    Tera::one_off(content.as_str(), &context, false)
        .map_err(|err| RenderError(RenderErrorKind::Tera(err)))
}

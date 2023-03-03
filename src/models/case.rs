use std::sync::Arc;
use std::time::Duration as StdDuration;
use humantime::format_duration;
use mongodb::bson::DateTime;
use twilight_http::Client;
use twilight_model::channel::embed::{Embed, EmbedAuthor, EmbedField, EmbedFooter};
use twilight_model::id::Id;
use twilight_model::id::marker::{GuildMarker, UserMarker};
use twilight_model::util::datetime::TimestampParseError;
use twilight_model::util::Timestamp;
use crate::RedisConnection;
use crate::utils::avatars::{DEFAULT_AVATAR, get_avatar_url, get_guild_icon_url};
use crate::utils::errors::Error;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Case {
    pub moderator_id: Id<UserMarker>,
    pub created_at: DateTime,
    pub guild_id: Id<GuildMarker>,
    pub member_id: Id<UserMarker>,
    pub action: CaseActionType,
    pub reason: Option<String>,
    pub removed: bool,
    pub duration: Option<i64>,
    pub index: u16
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(from = "u8", into = "u8")]
pub enum CaseActionType {
    Warn,
    Mute,
    Unmute,
    Ban,
    Unban,
    Kick,
    Timeout,
    Unknown(u8)
}

impl From<u8> for CaseActionType {
    fn from(action_type: u8) -> Self {
        match action_type {
            1 => CaseActionType::Warn,
            2 => CaseActionType::Mute,
            3 => CaseActionType::Unmute,
            4 => CaseActionType::Ban,
            5 => CaseActionType::Unban,
            6 => CaseActionType::Kick,
            7 => CaseActionType::Timeout,
            _ => CaseActionType::Unknown(action_type)
        }
    }
}

impl From<CaseActionType> for u8 {
    fn from(action_type: CaseActionType) -> Self {
        match action_type {
            CaseActionType::Warn => 1,
            CaseActionType::Mute => 2,
            CaseActionType::Unmute => 3,
            CaseActionType::Ban => 4,
            CaseActionType::Unban => 5,
            CaseActionType::Kick => 6,
            CaseActionType::Timeout => 7,
            CaseActionType::Unknown(action_type) => action_type
        }
    }
}

impl Case {
    pub fn to_dm_embed(&self, redis: &RedisConnection) -> Result<Embed, Error> {
        let guild = redis.get_guild(self.guild_id).map_err(Error::from)?;
        let guild_icon_url = get_guild_icon_url(guild.icon, self.guild_id);

        let author = EmbedAuthor {
            icon_url: Some(guild_icon_url),
            name: guild.name,
            proxy_icon_url: None,
            url: None
        };

        let mut embed = self.to_empty_embed(false, true).map_err(Error::from)?;
        embed.author = Some(author);
        Ok(embed)
    }

    pub async fn to_embed(&self, discord_http: Arc<Client>) -> Result<Embed, Error> {
        let moderator = match discord_http.user(self.moderator_id).exec().await {
            Ok(moderator) => moderator.model().await.ok(),
            Err(_) => None
        };

        let embed_author = moderator.map(|moderator| {
            let avatar = get_avatar_url(moderator.avatar, moderator.id);
            EmbedAuthor {
                icon_url: Some(avatar.clone()),
                name: format!("{}#{} {}", moderator.name, moderator.discriminator, moderator.id),
                proxy_icon_url: None,
                url: None
            }
        }).unwrap_or(EmbedAuthor {
            icon_url: Some(DEFAULT_AVATAR.to_string()),
            name: "Unknown#0000".to_string(),
            proxy_icon_url: Some(DEFAULT_AVATAR.to_string()),
            url: None
        });

        let mut embed = self.to_empty_embed(true, false).map_err(Error::from)?;
        embed.author = Some(embed_author);
        Ok(embed)
    }

    pub fn to_empty_embed(&self, member: bool, moderator: bool) -> Result<Embed, TimestampParseError> {
        let timestamp = Timestamp::from_secs(self.created_at.timestamp_millis() / 1000)?;

        let mut description = format!(
            "Action:** {:?}{}\n**Reason:** {}",
            self.action,
            self.duration.map(|duration| format!(
                "\n**Duration:** {}", format_duration(StdDuration::from_secs(duration as u64))
            )).unwrap_or_else(|| "".to_string()),
            self.reason.to_owned().unwrap_or_else(|| "None".to_string())
        );

        if moderator {
            description.insert_str(0, &*format!("**Moderator:** <@{}>**\n", self.moderator_id))
        }
        if member {
            description.insert_str(0, &*format!("**Member:** <@{}>**\n", self.member_id));
        }

        let footer = EmbedFooter {
            icon_url: None,
            proxy_icon_url: None,
            text: format!("Case #{}", self.index)
        };

        Ok(Embed {
            author: None,
            color: None,
            description: Some(description),
            fields: vec![],
            footer: Some(footer),
            image: None,
            kind: "".to_string(),
            provider: None,
            thumbnail: None,
            timestamp: Some(timestamp),
            title: None,
            url: None,
            video: None
        })
    }

    pub fn to_field(&self) -> EmbedField {
        let reason = self.reason.to_owned().unwrap_or_else(|| "None".to_string());
        EmbedField {
            inline: false,
            name: format!("**Case #{} - {:?}**", self.index, self.action),
            value: reason
        }
    }
}
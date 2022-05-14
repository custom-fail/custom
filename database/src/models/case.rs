use std::sync::Arc;
use std::time::Duration as StdDuration;
use mongodb::bson::DateTime;
use twilight_model::datetime::{Timestamp, TimestampParseError};
use serde::{Serialize, Deserialize};
use twilight_http::Client;
use twilight_model::channel::embed::{Embed, EmbedAuthor, EmbedField, EmbedFooter};
use twilight_model::id::Id;
use twilight_model::id::marker::{GuildMarker, UserMarker};
use humantime::format_duration;
use utils::avatars::{DEFAULT_AVATAR, get_avatar_url, get_guild_icon_url};
use utils::errors::Error;
use crate::redis::RedisConnection;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Case {
    pub moderator_id: Id<UserMarker>,
    pub created_at: DateTime,
    pub guild_id: Id<GuildMarker>,
    pub member_id: Id<UserMarker>,
    pub action: u8,
    pub reason: Option<String>,
    pub removed: bool,
    pub duration: Option<i64>,
    pub index: u16
}

fn action_type_to_string(action: u8) -> String {
    match action {
        1 => "Warn",
        2 => "Mute",
        3 => "Unmute",
        4 => "Ban",
        5 => "Unban",
        6 => "Kick",
        7 => "Timeout",
        _ => "Unknown"
    }.to_string()
}

impl Case {

    pub fn to_dm_embed(&self, redis: RedisConnection) -> Result<Embed, Error> {
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
        let moderator_member = discord_http.guild_member(self.guild_id, self.moderator_id)
            .exec().await.map_err(Error::from)?.model().await.ok();

        let embed_author = match moderator_member {
            Some(moderator) => {
                let avatar = get_avatar_url(moderator.user.avatar, moderator.user.id);
                EmbedAuthor {
                    icon_url: Some(avatar.clone()),
                    name: format!("{}#{} {}", moderator.user.name, moderator.user.discriminator, moderator.user.id),
                    proxy_icon_url: None,
                    url: None
                }
            },
            None => EmbedAuthor {
                icon_url: Some(DEFAULT_AVATAR.to_string()),
                name: "Unknown#0000".to_string(),
                proxy_icon_url: Some(DEFAULT_AVATAR.to_string()),
                url: None
            }
        };

        let mut embed = self.to_empty_embed(true, false).map_err(Error::from)?;
        embed.author = Some(embed_author);
        Ok(embed)
    }

    pub fn to_empty_embed(&self, member: bool, moderator: bool) -> Result<Embed, TimestampParseError> {

        let timestamp = Timestamp::from_secs(self.created_at.timestamp_millis() / 1000)?;

        let mut description = format!(
            "Action:** {}{}\n**Reason:** {}",
            action_type_to_string(self.action),
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

        let action = action_type_to_string(self.action);
        let reason = self.reason.to_owned().unwrap_or_else(|| "None".to_string());

        EmbedField {
            inline: false,
            name: format!("**Case #{} - {}**", self.index, action),
            value: reason
        }

    }
}
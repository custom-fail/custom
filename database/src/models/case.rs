use std::sync::Arc;
use mongodb::bson::DateTime;
use twilight_model::datetime::Timestamp;
use serde::{Serialize, Deserialize};
use twilight_http::Client;
use twilight_model::channel::embed::{Embed, EmbedAuthor, EmbedFooter};
use twilight_model::guild::Member;
use twilight_model::id::Id;
use twilight_model::id::marker::{GuildMarker, UserMarker};

#[derive(Serialize, Deserialize, Debug)]
pub struct Case {
    pub moderator_id: Id<UserMarker>,
    pub created_at: DateTime,
    pub guild_id: Id<GuildMarker>,
    pub member_id: Id<UserMarker>,
    pub action: u8,
    pub reason: Option<String>,
    pub removed: bool,
    pub duration: Option<String>,
    pub index: u16
}

fn get_avatar_from_member(member: Member) -> String {
    match member.user.avatar {
        Some(avatar) => {
            let file_format = if avatar.is_animated() { "gif" } else { "png" };
            format!("https://cdn.discordapp.com/avatars/{}/{}.{}", member.user.id, avatar, file_format)
        }
        None =>  "https://cdn.discordapp.com/embed/avatars/0.png".to_string()
    }
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
    pub async fn to_embed(&self, discord_http: Arc<Client>) -> Result<Embed, String> {

        let moderator_member = discord_http.guild_member(self.guild_id, self.moderator_id)
            .exec().await.map_err(|err| err.to_string())?.model().await.ok();

        let embed_author = match moderator_member {
            Some(moderator) => {
                let avatar = get_avatar_from_member(moderator.clone());
                EmbedAuthor {
                    icon_url: Some(avatar.clone()),
                    name: format!("{}#{}", moderator.user.name, moderator.user.discriminator),
                    proxy_icon_url: Some(avatar),
                    url: None
                }
            },
            None => EmbedAuthor {
                icon_url: Some("https://cdn.discordapp.com/embed/avatars/0.png".to_string()),
                name: "Unknown#0000".to_string(),
                proxy_icon_url: Some("https://cdn.discordapp.com/embed/avatars/0.png".to_string()),
                url: None
            }
        };

        let timestamp = Timestamp::from_secs(self.created_at.timestamp_millis() / 1000).map_err(|err| err.to_string())?;

        let description = format!("**Member:** <@{}>\n**Action:** {}\n**Reason:** {}",self.member_id, action_type_to_string(self.action),
            match &self.reason {
                Some(reason) => reason.clone().clone(),
                None => "None".to_string()
            }
        );
        
        let footer = EmbedFooter {
            icon_url: None,
            proxy_icon_url: None,
            text: format!("Case #{}", self.index)
        };

        Ok(Embed {
            author: Some(embed_author),
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
}
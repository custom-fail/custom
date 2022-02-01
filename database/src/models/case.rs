use twilight_model::datetime::Timestamp;
use serde::{Serialize, Deserialize};
use twilight_http::response::marker::MemberBody;
use twilight_model::channel::embed::{Embed, EmbedAuthor};
use twilight_model::guild::Member;
use twilight_model::id::Id;
use twilight_model::id::marker::{GuildMarker, UserMarker};
use twilight_model::util::ImageHash;

#[derive(Serialize, Deserialize, Debug)]
pub struct Case {
    pub moderator_id: Id<UserMarker>,
    pub created_at: Timestamp,
    pub guild_id: Id<GuildMarker>,
    pub member_id: Id<UserMarker>,
    pub action: u8,
    pub reason: Option<String>,
    pub removed: bool,
    pub duration: Option<String>,
    pub index: u16
}

fn get_avatar_from_member(member: Member) -> String {
    match member.avatar {
        Some(avatar) => {
            let file_format = if avatar.is_animated() { "gif" } else { "png" };
            format!("https://cdn.discordapp.com/avatars/{}/{}.{}", member.user.id, avatar, file_format)
        }
        None =>  "https://cdn.discordapp.com/embed/avatars/0.png".to_string()
    }
}
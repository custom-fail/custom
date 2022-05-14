use twilight_model::id::Id;
use twilight_model::id::marker::{GuildMarker, UserMarker};
use twilight_model::util::ImageHash;

pub const DEFAULT_AVATAR: &str = "https://cdn.discordapp.com/embed/avatars/0.png";
pub const DEFAULT_ICON: &str = "";

pub fn get_avatar_url(avatar: Option<ImageHash>, user_id: Id<UserMarker>) -> String {
    match avatar {
        Some(avatar) => {
            let file_format = if avatar.is_animated() { "gif" } else { "png" };
            format!("https://cdn.discordapp.com/avatars/{}/{}.{}", user_id, avatar, file_format)
        }
        None =>  DEFAULT_AVATAR.to_string()
    }
}

pub fn get_guild_icon_url(icon: Option<ImageHash>, guild_id: Id<GuildMarker>) -> String {
    match icon {
        Some(icon) => {
            let file_format = if icon.is_animated() { "gif" } else { "png" };
            format!("https://cdn.discordapp.com/icons/{}/{}.{}", guild_id, icon, file_format)
        }
        None =>  DEFAULT_ICON.to_string()
    }
}
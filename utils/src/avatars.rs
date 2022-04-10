use twilight_model::id::Id;
use twilight_model::id::marker::UserMarker;
use twilight_model::util::ImageHash;

pub fn get_avatar_url(avatar: Option<ImageHash>, user_id: Id<UserMarker>) -> String {
    match avatar {
        Some(avatar) => {
            let file_format = if avatar.is_animated() { "gif" } else { "png" };
            format!("https://cdn.discordapp.com/avatars/{}/{}.{}", user_id, avatar, file_format)
        }
        None =>  "https://cdn.discordapp.com/embed/avatars/0.png".to_string()
    }
}

use std::sync::Arc;
use twilight_http::Client;
use twilight_model::id::Id;
use twilight_model::id::marker::GuildMarker;
use twilight_model::util::Timestamp;
use crate::utils::errors::Error;

pub async fn run(
    guild_id: Id<GuildMarker>,
    joined_at: Option<Timestamp>,
    twilight_http: Arc<Client>
) -> Result<(), Error> {
    if let Some(joined_at) = joined_at {
        if chrono::Utc::now().timestamp_millis() - (joined_at.as_micros() / 1000) > 30 * 1000 {
            return Ok(())
        }
    } else { return Ok(()); };

    let application_id = twilight_http.current_user()
        .await.map_err(Error::from)?.model().await.map_err(Error::from)?.id;
    
    twilight_http
        .interaction(application_id.cast())
        .create_guild_command(guild_id)
        .chat_input(
            "setup", "Shows how and where you can setup the bot"
        ).map_err(Error::from)?.await.map_err(Error::from).map(|_| ())
}
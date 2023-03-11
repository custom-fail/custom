use std::sync::Arc;
use twilight_http::Client;
use twilight_model::gateway::event::Event;
use crate::context::Context;

pub mod automod;
pub mod case;
pub mod top;
pub mod cache;
pub mod restore;
pub mod setup;
pub mod interaction;

pub async fn on_event(
    event: Event,
    context: Arc<Context>,
    discord_http: Arc<Client>
) -> Result<(), ()> {
    match event {
        Event::MemberAdd(event) => {
            self::restore::mutes::run(event, discord_http, context).await.ok();
        }
        Event::BanRemove(event) => {
            self::restore::bans::run(event, &context.mongodb).await.ok();
        }
        Event::MessageCreate(event) => {
            let message = event.as_ref().0.to_owned();
            self::automod::run(message.to_owned(), discord_http, context.to_owned()).await.ok();
            self::top::run(message, context).await.ok();
        }
        Event::GuildCreate(event) => {
            tokio::spawn(self::setup::run(event.id, event.joined_at, discord_http));
            self::cache::on_guild_create(&context.redis, event).ok();
        },
        Event::GuildUpdate(event) => {
            self::cache::on_guild_update(&context.redis, event).ok();
        },
        Event::GuildDelete(event) => {
            self::cache::delete_guild(&context.redis, event.id).ok();
        },
        Event::RoleCreate(event) => {
            self::cache::fetch_and_set(&context.redis, discord_http, event.guild_id).await.ok();
        },
        Event::RoleUpdate(event) => {
            self::cache::fetch_and_set(&context.redis, discord_http, event.guild_id).await.ok();
        },
        Event::RoleDelete(event) => {
            self::cache::fetch_and_set(&context.redis, discord_http, event.guild_id).await.ok();
        },
        Event::GuildAuditLogEntryCreate(event) => {
            self::case::run(event, discord_http, context).await.ok();
        }
        Event::InteractionCreate(interaction) => {
            self::interaction::run(interaction, discord_http, context).await.ok();
        }
        _ => return Err(()),
    };
    Ok(())
}

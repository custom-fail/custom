use std::sync::Arc;
use twilight_http::Client;
use twilight_model::gateway::event::Event;
use twilight_model::gateway::payload::incoming::GuildCreate;
use crate::context::Context;
use crate::models::config::automod::TrigerEvent;

pub mod automod;
mod case;
mod top;
mod cache;
mod restore;
mod setup;
mod interaction;

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
            self::automod::run(message.to_owned(), discord_http, context.to_owned(), TrigerEvent::MessageCreate).await.ok();
            self::top::run(message, context).await.ok();
        }
        Event::MessageUpdate(event) => {
            self::automod::run(event.0, discord_http, context, TrigerEvent::MessageCreate).await.ok();
        }
        Event::GuildCreate(event) => {
            let guild = if let GuildCreate::Available(guild) = *event { guild } else { return Ok(()) };
            tokio::spawn(self::setup::run(guild.id, guild.joined_at, discord_http));
            self::cache::on_guild_create(&context.redis, guild).await.ok();
        },
        Event::GuildUpdate(event) => {
            self::cache::on_guild_update(&context.redis, event).await.ok();
        },
        Event::GuildDelete(event) => {
            self::cache::delete_guild(&context.redis, event.id).await.ok();
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

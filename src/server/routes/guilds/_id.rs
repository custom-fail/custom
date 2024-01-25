use std::collections::HashMap;
use std::sync::Arc;
use futures_util::{SinkExt, StreamExt, TryFutureExt};
use twilight_model::guild::Permissions;
use twilight_model::id::Id;
use twilight_model::id::marker::GuildMarker;
use warp::Filter;
use futures_util::FutureExt;
use futures_util::stream::SplitSink;
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use twilight_model::user::CurrentUserGuild;
use warp::ws::{Message, WebSocket, Ws};
use crate::context::Context;
use crate::{response_type, with_value};
use crate::server::error::{MapErrorIntoInternalRejection, Rejection};
use crate::server::session::{Authenticator, AuthorizationInformation, authorize_user, Sessions};

type GuildId = Id<GuildMarker>;

pub fn run(
    context: Arc<Context>,
    authenticator: Arc<Authenticator>,
    sessions: Arc<Sessions>
) -> response_type!() {
    let with_context = with_value!(context);

    warp::path!("guilds" / GuildId)
        .and(authorize_user(authenticator, sessions))
        .and(with_context)
        .and_then(check_guild)
        .and(warp::ws())
        .map(|(info, guild): (Arc<AuthorizationInformation>, CurrentUserGuild), ws: Ws| {
            ws.on_upgrade(move |ws| {
                handle_connection(ws, info, guild)
            })
        })
}

const MINIMAL_REQUIRED_PERMISSIONS: Permissions = Permissions::MANAGE_GUILD;

async fn check_guild(
    guild_id: Id<GuildMarker>,
    info: Arc<AuthorizationInformation>,
    context: Arc<Context>
) -> Result<(Arc<AuthorizationInformation>, CurrentUserGuild), warp::Rejection> {
    let guild = info.http.current_user_guilds()
        .await.map_rejection()?.model().await.map_rejection()?
        .into_iter().find(|guild| {
            guild.owner || guild.permissions.intersects(MINIMAL_REQUIRED_PERMISSIONS)
        }).ok_or(Rejection::NotMutualGuild)?;

    let is_mutual = context.redis.guild_exists(guild_id).await.map_rejection()?;
    if !is_mutual { return err!(Rejection::NotMutualGuild) }

    Ok((info, guild))
}

macro_rules! close {
    ($tx: expr) => {
        let _ = $tx.close().await;
    };
}

macro_rules! unwrap_or_close_and_return {
    ($target: expr, $tx: expr) => {
        match $target {
            Ok(value) => value,
            Err(_) => {
                close!($tx);
                return
            }
        }
    };
}

async fn handle_connection(
    ws: WebSocket,
    info: Arc<AuthorizationInformation>,
    guild: CurrentUserGuild
) {
    let (mut tx, mut rx) = ws.split();

    while let Some(result) = rx.next().await {
        let message = match result {
            Ok(message) => message,
            Err(_) => {
                close!(tx);
                break
            }
        };

        if !message.is_text() {
            close!(tx);
            break
        }

        on_message(message, &info, &guild, &mut tx).await;
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "action", content = "data")]
enum InboundMessage {
    GuildConfigUpdate(HashMap<String, Value>),
    ApplyChanges
}

async fn on_message(
    message: Message,
    _info: &Arc<AuthorizationInformation>,
    _guild: &CurrentUserGuild,
    tx: &mut SplitSink<WebSocket, Message>
) {
    let message = unwrap_or_close_and_return!(message.to_str(), tx);
    let message: InboundMessage = unwrap_or_close_and_return!(
        serde_json::from_str(message), tx
    );

    match message {
        InboundMessage::GuildConfigUpdate(_) => {}
        InboundMessage::ApplyChanges => {}
    }

    ()
}

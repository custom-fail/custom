use std::borrow::Cow;
use std::sync::Arc;
use futures_util::{SinkExt, StreamExt};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::mpsc::UnboundedSender;
use tokio_stream::wrappers::UnboundedReceiverStream;
use twilight_model::id::Id;
use twilight_model::id::marker::UserMarker;
use twilight_model::user::CurrentUserGuild;
use warp::ws::{Message, WebSocket};
use crate::context::Context;
use crate::database::redis::PartialGuild;
use crate::models::config::GuildConfig;
use crate::ok_or_return;
use crate::server::guild::editing::GuildsEditing;
use crate::server::session::AuthorizationInformation;

macro_rules! close {
    ($tx: expr, $reason: expr) => {
        let _ = $tx.send(OutboundAction::Close($reason));
    };
}

macro_rules! unwrap_or_close_and_return {
    ($target: expr, $tx: expr, $reason: expr) => {
        match $target {
            Ok(value) => value,
            Err(_) => {
                close!($tx, $reason);
                return
            }
        }
    };
}

pub enum CloseReason {
    MessageIsNotString,
    CannotParseJSON
}

impl CloseReason {
    pub fn code(&self) -> u16 {
        match self {
            CloseReason::MessageIsNotString => 4001,
            CloseReason::CannotParseJSON => 4002,
        }
    }

    pub fn text(&self) -> impl Into<Cow<'static, str>> {
        match self {
            CloseReason::MessageIsNotString => "Message is not UTF-8 string",
            CloseReason::CannotParseJSON => "Cannot parse JSON message"
        }
    }
}

pub struct Connection {
    pub user_id: Id<UserMarker>,
    pub session_id: ObjectId,
    pub tx: UnboundedSender<OutboundAction>
}

pub async fn handle_connection(
    context: Arc<Context>,
    ws: WebSocket,
    info: Arc<AuthorizationInformation>,
    guild: CurrentUserGuild,
    guilds_editing: Arc<GuildsEditing>
) {
    let (mut ws_tx, mut ws_rx) = ws.split();

    let (tx, rx) =
        tokio::sync::mpsc::unbounded_channel();

    let mut rx = UnboundedReceiverStream::new(rx);

    tokio::spawn(async move {
        while let Some(message) = rx.next().await {
            match message {
                OutboundAction::Message(msg) => {
                    if let Ok(data) = serde_json::to_string(&msg) {
                        let _ = ws_tx.send(Message::text(data)).await;
                    }
                }
                OutboundAction::Close(reason) => {
                    let _ = ws_tx.send(
                        Message::close_with(reason.code(), reason.text())
                    ).await;
                }
            }
        }
        let _ = ws_tx.close().await;
    });

    let session_id = ObjectId::new();
    let guild_id = guild.id;

    guilds_editing.add_connection(guild_id, Connection {
        user_id: info.user.id,
        session_id,
        tx: tx.to_owned(),
    }).await;

    let _ = tx.send(OutboundAction::Message(OutboundMessage::Initialization {
        cached: ok_or_return!(context.redis.get_guild(guild.id).await, Ok),
        oauth2: guild.to_owned(),
        session_id
    }));

    while let Some(result) = ws_rx.next().await {
        let message = match result {
            Ok(message) => message,
            Err(_) => {
                break
            }
        };

        if !message.is_text() {
            break
        }

        on_message(message, &info, &guild, &tx, &guilds_editing, &context).await;
    }

    guilds_editing.remove_connection(guild_id, session_id).await;
}
#[derive(Debug, Deserialize)]
#[serde(tag = "action", content = "data")]
enum InboundMessage {
    GuildConfigUpdate(Value),
    ApplyChanges
}

#[derive(Debug, Serialize)]
#[serde(tag = "action", content = "data")]
pub enum OutboundMessage {
    Initialization {
        oauth2: CurrentUserGuild,
        cached: PartialGuild,
        session_id: ObjectId
    },
    UpdateConfigurationData {
        saved_config: GuildConfig,
        changes: Value,
        users: Vec<Id<UserMarker>>
    }
}

pub enum OutboundAction {
    Message(OutboundMessage),
    Close(CloseReason)
}

async fn on_message(
    message: Message,
    info: &Arc<AuthorizationInformation>,
    guild: &CurrentUserGuild,
    tx: &UnboundedSender<OutboundAction>,
    guilds_editing: &Arc<GuildsEditing>,
    context: &Arc<Context>
) {
    let message = unwrap_or_close_and_return!(
        message.to_str(), tx, CloseReason::MessageIsNotString
    );

    let message: InboundMessage = unwrap_or_close_and_return!(
        serde_json::from_str(message), tx, CloseReason::CannotParseJSON
    );

    match message {
        InboundMessage::GuildConfigUpdate(changes) => {
            let _ = guilds_editing.marge_changes(info.user.id, guild.id, changes).await;
            guilds_editing.broadcast_changes(context, guild.id).await;
        }
        InboundMessage::ApplyChanges => {}
    }
}

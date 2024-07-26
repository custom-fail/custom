use std::sync::Arc;

use futures_util::StreamExt;
use twilight_model::guild::Permissions;
use twilight_model::id::Id;
use twilight_model::id::marker::GuildMarker;
use twilight_model::user::CurrentUserGuild;
use warp::Filter;
use warp::ws::Ws;

use crate::{response_type, with_value};
use crate::context::Context;
use crate::server::error::{MapErrorIntoInternalRejection, Rejection};
use crate::server::guild::editing::GuildsEditing;
use crate::server::guild::ws::handle_connection;
use crate::server::session::{Authenticator, AuthorizationInformation, authorize_user, Sessions};

type GuildId = Id<GuildMarker>;

pub fn run(
    context: Arc<Context>,
    authenticator: Arc<Authenticator>,
    sessions: Arc<Sessions>
) -> response_type!() {
    let with_context = with_value!(context);

    let guilds_editing = Arc::new(GuildsEditing::default());
    let with_guilds_editing = with_value!(guilds_editing);

    warp::path!("guilds" / GuildId)
        .and(authorize_user(authenticator, sessions))
        .and(with_context.clone())
        .and_then(check_guild)
        .and(warp::ws())
        .and(with_context)
        .and(with_guilds_editing)
        .map(|
            (info, guild): (Arc<AuthorizationInformation>, CurrentUserGuild),
            ws: Ws,
            context: Arc<Context>,
            guilds_editing: Arc<GuildsEditing>
        | {
            let context = context.clone();
            let guilds_editing = guilds_editing.clone();
            ws.on_upgrade(move |ws| {
                handle_connection(context, ws, info, guild, guilds_editing)
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

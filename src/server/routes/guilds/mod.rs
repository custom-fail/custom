use std::sync::Arc;
use futures_util::future::join_all;
use futures_util::{FutureExt, StreamExt};
use redis::{AsyncCommands, RedisError, ToRedisArgs};
use serde::{Deserialize, Serialize};
use twilight_model::id::Id;
use twilight_model::id::marker::GuildMarker;
use twilight_model::user::CurrentUserGuild;
use warp::{Filter, Reply};
use warp::hyper::body::HttpBody;
use crate::context::Context;
use crate::{response_type, with_value};
use crate::server::error::MapErrorIntoInternalRejection;
use crate::server::session::{Authenticator, AuthorizationInformation, authorize_user, Sessions};

mod _id;

#[derive(Serialize, Deserialize)]
struct Response {
    guilds: Vec<CurrentUserGuild>,
    mutual: Vec<Id<GuildMarker>>
}

pub fn list(context: Arc<Context>, authenticator: Arc<Authenticator>, sessions: Arc<Sessions>) -> response_type!() {
    let with_context = with_value!(context);

    warp::get()
        .and(warp::path("guilds"))
        .and(authorize_user(authenticator, sessions))
        .and(with_context)
        .and_then(run)
}

async fn run(
    info: Arc<AuthorizationInformation>,
    context: Arc<Context>
) -> Result<impl Reply, warp::Rejection> {
    let guilds = info.http.current_user_guilds()
        .await.map_rejection()?.model().await.map_rejection()?;

    let ids: Vec<Id<GuildMarker>> = guilds
        .iter().map(|guild| guild.id).collect();

    let mut pipe = redis::pipe().atomic().to_owned();
    ids.iter().for_each(|id| { pipe.exists(format!("guilds.{id}")); });

    let result: Vec<bool> = pipe.query_async(
        &mut context.redis.client.get_async_connection().await.map_rejection()?
    ).await.map_rejection()?;

    let mutual = result.into_iter().enumerate()
        .filter_map(|(i, exists)| if exists { Some(ids[i]) } else { None })
        .collect();

    Ok(warp::reply::json(&Response { guilds, mutual }))
}
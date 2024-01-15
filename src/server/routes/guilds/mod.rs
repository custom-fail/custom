use std::sync::Arc;
use futures_util::future::join_all;
use warp::{Filter, Reply};
use crate::context::Context;
use crate::{response_type, with_value};
use crate::server::error::MapErrorIntoInternalRejection;
use crate::server::session::{Authenticator, AuthorizationInformation, authorize_user, Sessions};

mod _id;

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
    // join_all(guilds.map(|guild| context.redis.)
    Ok(warp::reply::json(&guilds))
}
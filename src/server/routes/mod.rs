use std::sync::Arc;
use reqwest::StatusCode;
use twilight_http::Client;
use warp::Filter;
use crate::context::Context;
use crate::{all_macro, response_type};

#[cfg(feature = "http-interactions")]
mod interactions;

all_macro!(
    cfg(feature = "api");
    mod login;
    mod guilds;
);

#[cfg(feature = "api")]
mod users {
    pub mod me;
}

pub fn get_all_routes(
    discord_http: Arc<Client>,
    context: Arc<Context>,
    #[cfg(feature = "http-interactions")] public_key: ed25519_dalek::PublicKey
) -> response_type!() {
    let filter = warp::path::end().map(|| {
        warp::reply::with_status("👀", StatusCode::OK)
    });

    #[cfg(feature = "http-interactions")]
    let filter = filter.or(interactions::filter(
        discord_http, context.to_owned(), public_key
    ));

    #[cfg(feature = "api")]
    let (sessions, authenticator) = {
        (
            Arc::new(crate::server::session::Sessions::default()),
            Arc::new(crate::server::session::Authenticator::with_random_key())
        )
    };

    #[cfg(feature = "api")]
    let filter = filter
        .or(login::login(authenticator.to_owned(), sessions.to_owned()))
        .or(users::me::run(authenticator.to_owned(), sessions.to_owned()))
        .or(guilds::list(context, authenticator, sessions));

    filter
}